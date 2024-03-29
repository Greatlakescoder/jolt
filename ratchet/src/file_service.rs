use anyhow::{Ok, Result};
use rayon::prelude::*;
use std::sync::Arc;
use std::{
    ffi::OsString,
    fs::{self, DirEntry},
    sync::Mutex,
};


pub struct GrepRequest<'a> {
    pub path: &'a str,
    pub search_term: &'a str,
    pub show_full_path: bool,
}

pub fn grep<'a>(
    request: GrepRequest,
    storage: Arc<Mutex<Vec<String>>>,
) -> Result<Arc<Mutex<Vec<String>>>> {
    let dir = fs::read_dir(request.path)?;
    let entries: Vec<DirEntry> = dir.filter_map(Result::ok).collect();

    entries.par_iter().try_for_each(|file| {
        if file.path().is_dir() {
            grep(
                GrepRequest {
                    path: file.path().to_str().unwrap(),
                    search_term: request.search_term,
                    show_full_path: request.show_full_path,
                },
                storage.clone(),
            )?;
        } else if file
            .file_name()
            .to_str()
            .unwrap()
            .contains(request.search_term)
        {
            let mut storage = storage.lock().unwrap();
            if request.show_full_path {
                storage.push(file.path().as_os_str().to_string_lossy().to_string());
            } else {
                storage.push(file.file_name().into_string().unwrap());
            }
           
        }
        Ok(())
    })?;

    return Ok(storage);
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
    pub folder_size: usize,
}

impl Folder {
    pub fn new(name: String, size: usize) -> Folder {
        Folder {
            name,
            files: Vec::new(),
            folder_size: size,
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
        for f in self.files {
            println!(
                "File Name: {}\nFile Size in Mb {}\n",
                f.name.to_str().unwrap(),
                f.size
            );
        }
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

pub fn find_largest_files(path: &str, mut folder: Folder) -> Result<Folder> {
    let entries = fs::read_dir(path)?;
    for file in entries {
        let f = file?;
        if f.path().is_dir() {
            folder = find_largest_files(f.path().to_str().unwrap(), folder)?;
        } else {
            let size_in_mb = f.metadata().unwrap().len() / 1024 / 1024;
            folder.add_file(LargeFile::new(f.path().into_os_string(), size_in_mb));
        }
    }
    return Ok(folder);
}
