use anyhow::{Ok, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::clone;
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

#[derive(Clone, Serialize, Deserialize, PartialEq,Debug)]
pub struct LargeFile {
    pub filename: String,
    pub file_size: u64,
}

pub fn find_largest_files(
    path: &str,
    storage: Arc<Mutex<Vec<LargeFile>>>,
) -> Result<Arc<Mutex<Vec<LargeFile>>>> {
    let dir = fs::read_dir(path)?;
    let entries: Vec<DirEntry> = dir.filter_map(Result::ok).collect();
    entries.par_iter().try_for_each(|file| {
        if file.path().is_dir() {
            find_largest_files(file.path().to_str().unwrap(), storage.clone())?;
        } else {
            match file.metadata() {
                std::result::Result::Ok(metadata) => {
                    let size_in_mb = metadata.len() / 1024 / 1024;
                    let large_file = LargeFile {
                        filename: file.file_name().into_string().expect("Hmmmmm"),
                        file_size: size_in_mb,
                    };

                    let mut tmp_storage = storage.lock().unwrap();
                    if tmp_storage.len() == 10 {
                        replace_smallest_file(large_file, &mut tmp_storage);
                    } else {
                        tmp_storage.push(large_file)
                    }
                }
                Err(err) => {
                    // Handle the error here, or simply skip the file
                    println!("Failed during find_largest file {:?}", err);
                }
            }
        }

        Ok(())
    })?;

    return Ok(storage);
}

fn replace_smallest_file(file: LargeFile, vault: &mut Vec<LargeFile>) {
    let mut smallest_file = vault[0].clone();
    for f in vault.clone().into_iter() {
        if f.file_size < smallest_file.file_size {
            smallest_file = f.clone();
        }
    }
    let index = vault.iter().position(|x| *x == smallest_file);
    match index {
        Some(i) => {
            vault[i] = file;
        }
        None => {
            println!("Failed to replace smallest file")
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;
    #[test]
    fn test_replace_smallest_file() {
        let mut vault = vec![
            LargeFile {
                filename: "file1.txt".to_string(),
                file_size: 10,
            },
            LargeFile {
                filename: "file2.txt".to_string(),
                file_size: 5,
            },
            LargeFile {
                filename: "file3.txt".to_string(),
                file_size: 8,
            },
        ];

        let new_file = LargeFile {
            filename: "file4.txt".to_string(),
            file_size: 7,
        };

        replace_smallest_file(new_file, &mut vault);

        assert_eq!(vault.len(), 3);
        assert_eq!(vault[0].filename, "file4.txt");
        assert_eq!(vault[0].file_size, 7);
    }
    #[test]
    fn test_find_largest_files() {
        let storage = Arc::new(Mutex::new(Vec::new()));

        // Create a temporary directory for testing
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap();

        // Create some test files with different sizes
        let file1 = temp_dir_path.to_owned() + "/file1.txt";
        let file2 = temp_dir_path.to_owned() + "/file2.txt";
        let file3 = temp_dir_path.to_owned() + "/file3.txt";
        let file4 = temp_dir_path.to_owned() + "/file4.txt";

        std::fs::write(&file1, "This is file 1").unwrap();
        std::fs::write(&file2, "This is file 2").unwrap();
        std::fs::write(&file3, "This is file 3").unwrap();
        std::fs::write(&file4, "This is file 4").unwrap();

        // Call the function under test
        find_largest_files(temp_dir_path, storage.clone()).unwrap();

        // Check the contents of the storage
        let storage = storage.lock().unwrap();
        assert_eq!(storage.len(), 3);
        assert_eq!(storage[0].filename, "file1.txt");
        assert_eq!(storage[0].file_size, 0); // File size is 0 because we didn't set it in the test
        assert_eq!(storage[1].filename, "file2.txt");
        assert_eq!(storage[1].file_size, 0); // File size is 0 because we didn't set it in the test
        assert_eq!(storage[2].filename, "file3.txt");
        assert_eq!(storage[2].file_size, 0); // File size is 0 because we didn't set it in the test
    }

    #[test]
    fn test_grep() {
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_str().unwrap();

        // Create some test files
        let file1_path = temp_dir_path.to_owned() + "/file1.txt";
        let file2_path = temp_dir_path.to_owned() + "/file2.txt";
        let file3_path = temp_dir_path.to_owned() + "/file3.txt";

        fs::write(&file1_path, "Hello, world!").unwrap();
        fs::write(&file2_path, "This is a test").unwrap();
        fs::write(&file3_path, "Another test").unwrap();

        let request = GrepRequest {
            path: temp_dir_path,
            search_term: "test",
            show_full_path: true,
        };

        let storage = Arc::new(Mutex::new(Vec::new()));
        let result = grep(request, storage.clone()).unwrap();

        let result_storage = result.lock().unwrap();
        assert_eq!(result_storage.len(), 2);
        assert!(result_storage.contains(&file2_path.to_owned()));
        assert!(result_storage.contains(&file3_path.to_owned()));
    }
}
