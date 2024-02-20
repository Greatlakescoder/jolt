use core::fmt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{ffi::OsString, fs, io};
use sysinfo::{Disks, System};

#[derive(PartialEq, Clone)]
pub struct LargeFile {
    pub name: String,
    pub size: u64,
}

impl LargeFile {
    pub fn new(name: String, size: u64) -> LargeFile {
        LargeFile { name, size }
    }
}
#[derive(PartialEq, Clone)]
pub struct Folder {
    pub name: String,
    pub files: Vec<LargeFile>,
    pub folder_size: usize,
}

impl Folder {
    pub fn new(name: String, folder_size: usize) -> Folder {
        Folder {
            name,
            files: Vec::new(),
            folder_size: folder_size,
        }
    }

    fn add_file(&mut self, file: LargeFile) {
        if self.files.len() == self.folder_size {
            self.replace_smallest_file(file)
        } else {
            self.files.push(file);
        }
    }

    fn replace_smallest_file(&mut self, file: LargeFile) {
        let mut smallest_file = self.files[0].clone();
        for f in &self.files {
            if f.size < smallest_file.size {
                smallest_file = f.clone();
            }
        }
        let index = self.files.iter().position(|x| *x == smallest_file).unwrap();
        self.files[index] = file;
    }
    pub fn sort_files(&mut self) {
        self.files.sort_by(|a, b| {
            b.size
                .partial_cmp(&a.size)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn print_files(self) {
        println!("----------- Files in {} -----------\n", self.name);
        for f in self.files {
            println!("File Name: {}\nFile Size in Mb {}\n", f.name, f.size);
        }
    }
}

pub fn grep(path: &str, search_term: &str) {
    let entries = fs::read_dir(path);
    match entries {
        Ok(dir_entries) => {
            for file in dir_entries {
                match file {
                    Ok(f) => {
                        if f.path().is_dir() {
                            grep(f.path().to_str().unwrap(), search_term);
                        } else {
                            if f.file_name().to_str().unwrap().contains(search_term) {
                                println!("File Name: {}", f.file_name().to_str().unwrap())
                            }
                        }
                    }
                    Err(err) => {
                        println!("Cannot read {} {}", path, err);
                    }
                }
            }
        }
        Err(err) => {
            println!("Cannot read {} {}", path, err);
        }
    }
}

/*
   This is the main function, we want to find the largest files on a hostmachine and report that back to the user
   - - - - - - - Steps - - - - - - - - -
   1. If they dont pass use a directory we should find the largest file across avaliable disks - default behavior
   2. Once we have the avaliable disks we create threads per disks to determine the largest files on those disks
   3. We want to report to the user the progress as we process the directories
       a. As of 2/17 this will just be updated as we finsh one level down from parent directory
   4. Result will be top 10 largest files per disk
*/

struct SearchEngine {
    disks_to_search: Vec<PathBuf>,
    tx: Sender<PathBuf>,
    rx: Receiver<PathBuf>,
    thread_pool: Arc<Mutex<i32>>,
}

impl SearchEngine {
    fn new(defined_path: &str) -> SearchEngine {
        let (tx, rx) = mpsc::channel();
        SearchEngine {
            disks_to_search: get_avaliable_disks(defined_path),
            tx,
            rx,
            thread_pool: Arc::new(Mutex::new(0)),
        }
    }
}
fn get_avaliable_disks(defined_path: &str) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = vec![];
    if defined_path == "" {
        let disks = Disks::new_with_refreshed_list();
        for disk in disks.list() {
            paths.push(disk.mount_point().to_path_buf());
        }
    } else {
        paths.push(PathBuf::from(defined_path))
    }
    paths
}

pub fn search_engine(defined_path: &str) {
    let engine = SearchEngine::new(defined_path);
    println!("Search starting at {}", defined_path);
    let mut disk_handles: Vec<JoinHandle<()>> = vec![];
    let mut folder = Folder::new("Test".to_string(), 10);
    for p in engine.disks_to_search {
        if p.starts_with("/usr/lib/wsl/drivers")
            || p.starts_with("/usr/lib/wsl/lib")
            || p.starts_with("/mnt/wslg/distro")
            || p.starts_with("/mnt/wslg/doc")
        {
            continue;
        }
        let tx = engine.tx.clone();
        let thread_pooler = engine.thread_pool.clone();
        let dh: JoinHandle<()> = thread::spawn(move || {
            find_directories_for_path(&p, tx, thread_pooler);
        });
        disk_handles.push(dh);
    }

    /*
       We need to do a recv timeout so we are not waiting infinetly since engine.rc.recv will block
    */
    while let Ok(file_received) = engine.rx.recv_timeout(Duration::from_secs(5)) {
        let size_in_mb = file_received.metadata();
        match size_in_mb {
            Ok(meta) => {
                if meta.is_file() {
                    let size_in_mb = meta.len() / 1024 / 1024;
                    folder.add_file(LargeFile::new(
                        file_received.to_str().unwrap().to_string(),
                        size_in_mb,
                    ));
                }
            }
            Err(err) => {
                println!("Cannot read {:?} {}", file_received, err);
            }
        }
    }
    folder.print_files()
}
/*
   This function will find any directories for the given path, if there are also files in here it will
   send them to a channel to be processed, essentially this function should only be trying to find directories
*/
fn find_directories_for_path(path: &Path, tx: Sender<PathBuf>, thread_pool: Arc<Mutex<i32>>) {
    let entries = fs::read_dir(path);
    match entries {
        Ok(dir_entries) => {
            for file in dir_entries {
                match file {
                    Ok(f) => {
                        let directory_sender = tx.clone();
                        let mut thread_count = thread_pool.lock().unwrap();
                        if f.path().is_dir() {
                            if *thread_count < 30 {
                                *thread_count += 1;
                                drop(thread_count); // Drop the lock before spawning the thread

                                let thread_pooler = thread_pool.clone();
                                let jh = thread::spawn(move || {
                                    find_directories_for_path(
                                        &f.path(),
                                        directory_sender,
                                        thread_pooler.clone(),
                                    );
                                    let mut thread_count = thread_pooler.lock().unwrap();
                                    *thread_count -= 1;
                                });
                            } else {
                                drop(thread_count);
                                find_directories_for_path(
                                    &f.path(),
                                    directory_sender,
                                    thread_pool.clone(),
                                );
                            }
                        } else {
                            directory_sender.send(f.path()).unwrap();
                        }
                    }
                    Err(err) => {
                        println!("Cannot read {:?} {}", path, err);
                    }
                }
            }
        }
        Err(err) => {
            println!("Cannot read {:?} {}", path, err);
        }
    }
}

// pub fn find_largest_files(path: &str, mut folder: Folder) -> Folder {
//     // If path is blank/empty we need to show directories instead of files
//     if path == "" {
//         find_largest_directories(path, folder.clone());
//         return folder;
//     }
//     let entries = fs::read_dir(path);
//     match entries {
//         Ok(dir_entries) => {
//             for file in dir_entries {
//                 match file {
//                     Ok(f) => {
//                         if f.path().is_dir() {
//                             folder = find_largest_files(f.path().to_str().unwrap(), folder);
//                         } else {
//                             let size_in_mb = f.metadata().unwrap().len() / 1024 / 1024;
//                             folder.add_file(LargeFile::new(f.path().into_os_string(), size_in_mb));
//                         }
//                     }
//                     Err(err) => {
//                         println!("Cannot read {} {}", path, err);
//                     }
//                 }
//             }
//         }
//         Err(err) => {
//             println!("Cannot read {} {}", path, err);
//         }
//     }

//     return folder;
// }
