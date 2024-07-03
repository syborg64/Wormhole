use fuser::FileType;
use std::fs;
use std::{collections::HashMap, io::Error, os::unix::fs::MetadataExt, path::Path};
use walkdir::WalkDir;

const COPIED_ROOT: &str = "./original";

// DEV only

fn original_ino() -> u64 {
    let meta = fs::metadata(Path::new(COPIED_ROOT)).unwrap();
    meta.ino()
}

// Axel's territory

fn index_folder(pth: &Path) -> HashMap<u64, String> {
    let mut arbo: HashMap<u64, String> = HashMap::new();
    arbo.insert(1, pth.to_string_lossy().to_string());

    for entry in WalkDir::new(COPIED_ROOT).into_iter().filter_map(|e| e.ok()) {
        println!("{}", entry.path().display());
    }
    arbo
}

/////

fn file_inode(file_name: &str) -> Result<(u64, FileType, String), Error> {
    let metadata = fs::metadata(file_name)?;
    Ok((metadata.ino(), metadata.file_type(), file_name.to_owned()))
}

pub fn list_files(path: &str) -> Result<Vec<(u64, FileType, String)>, Error> {
    // let files: Vec<(u64, FileType, String)>;

    files = fs::read_dir(path)?.map(|entry| file_inode(entry));

    // match fs::read_dir(path) {
    //     Ok(entries) => {
    //         for entry in entries {
    //             match entry {
    //                 Ok(entry) => match file_inode(entry) {
    //                     Ok(file_inode) => files.append(file_inode),
    //                     Err(e) => return (e),
    //                 },
    //                 Err(e) => return (e),
    //             }
    //         }
    //     }
    //     Err(e) => return (e),
    // }
}
