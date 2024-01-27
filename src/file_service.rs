use std::{ffi::OsString, fs, io, os::unix::fs::MetadataExt};

pub fn get_files_in_directory(path: &str) {
    // Get a list of all entries in the folder
    let entries = fs::read_dir(path).unwrap();

    // Extract the filenames from the directory entries and store them in a vector
    for file in entries {
        match file {
            Ok(f) => {
                if f.path().is_dir() {
                    get_files_in_directory(f.path().to_str().unwrap());
                } else {
                    let size_in_mb = f.metadata().unwrap().size() / 1024 / 1024;
                    println!(
                        "File Name: {}, Size in Mb {}",
                        f.file_name().to_str().unwrap(),
                        size_in_mb
                    )
                }
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }
}

pub fn grep(path: &str, search_term: &str) {
    let entries = fs::read_dir(path).unwrap();

    // Extract the filenames from the directory entries and store them in a vector
    for file in entries {
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
                println!("{}", err);
            }
        }
    }
}
#[derive(PartialEq, Clone)]
pub struct LargeFile {
    pub name: OsString,
    pub size: u64,
}

impl LargeFile {
    pub fn new(name: OsString, size: u64) -> LargeFile {
        LargeFile { name, size }
    }
}
#[derive(PartialEq, Clone)]
pub struct Folder {
    pub name: String,
    pub files: Vec<LargeFile>,
}

impl Folder {
    pub fn new(name: String) -> Folder {
        Folder {
            name,
            files: Vec::new(),
        }
    }

    fn add_file(&mut self, file: LargeFile) {
        if self.files.len() == 10 {
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
}

struct FileCabinet {
    folders: Vec<Folder>,
}

impl FileCabinet {
    fn new() -> FileCabinet {
        FileCabinet {
            folders: Vec::new(),
        }
    }

    fn add_folder(&mut self, folder: Folder) {
        self.folders.push(folder);
    }
}

pub fn find_largest_files(path: &str, mut folder: Folder) -> Folder {
    let entries = fs::read_dir(path).unwrap();
    for file in entries {
        match file {
            Ok(f) => {
                if f.path().is_dir() {
                    folder = find_largest_files(f.path().to_str().unwrap(), folder);
                } else {
                    let size_in_mb = f.metadata().unwrap().size() / 1024 / 1024;
                    folder.add_file(LargeFile::new(f.file_name(), size_in_mb));
                }
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }
    return folder;
}
