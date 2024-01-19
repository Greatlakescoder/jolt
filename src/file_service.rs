use std::{fs, io, os::unix::fs::MetadataExt};
use clap::Parser;

// /// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,

    #[arg(short,long,default_value_t = String::from("."))]
    path: String,
}


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
                    println!("File Name: {}, Size in Mb {}", f.file_name().to_str().unwrap(), size_in_mb)
                }
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }
}

// fn main() {
//     let args = Args::parse();

//     for _ in 0..args.count {
//         println!("Hello {}!", args.name)
//     }
//     get_files_in_directory(&args.path);
// }
