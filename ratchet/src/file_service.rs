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

#[derive(Clone, Serialize, Deserialize, PartialEq)]
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
            let size_in_mb = file.metadata().unwrap().len() / 1024 / 1024;
            let large_file = LargeFile {
                filename: file.file_name().into_string().unwrap(),
                file_size: size_in_mb,
            };

            let mut tmp_storage = storage.lock().unwrap();
            if tmp_storage.len() == 10 {
                replace_smallest_file(large_file, &mut tmp_storage);
            } else {
                tmp_storage.push(large_file)
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
    let index = vault.iter().position(|x| *x == smallest_file).unwrap();
    vault[index] = file;
}
