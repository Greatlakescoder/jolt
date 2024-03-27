use std::{ffi::OsString, fs, io};

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
                    let size_in_mb = f.metadata().unwrap().len() / 1024 / 1024;
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
                        println!("Cannot read {} {}", path,err);
                    }
                }
            }
        }
        Err(err) => {
            println!("Cannot read {} {}",path, err);
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
    pub folder_size: usize,
}

impl Folder {
    pub fn new(name: String, size: usize) -> Folder {
        Folder {
            name,
            files: Vec::new(),
            folder_size: size
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
        };
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
    let entries = fs::read_dir(path);
    match entries {
        Ok(dir_entries) => {
            for file in dir_entries {
                match file {
                    Ok(f) => {
                        if f.path().is_dir() {
                            folder = find_largest_files(f.path().to_str().unwrap(), folder);
                        } else {
                            let size_in_mb = f.metadata().unwrap().len() / 1024 / 1024;
                            folder.add_file(LargeFile::new(f.path().into_os_string(), size_in_mb));
                        }
                    }
                    Err(err) => {
                        println!("Cannot read {} {}", path,err);
                    }
                }
            }
        }
        Err(err) => {
            println!("Cannot read {} {}",path, err);
        }
    }

    return folder;
}
