use fuser::FileType;
use std::fs;
use std::ptr::metadata;
use std::{collections::HashMap, io::Error, os::unix::fs::MetadataExt, path::Path};
use walkdir::WalkDir;

const COPIED_ROOT: &str = "./original/";

fn original_ino() -> u64 {
    let meta = fs::metadata(Path::new(COPIED_ROOT)).unwrap();
    meta.ino()
}

pub fn index_folder(pth: &Path) -> HashMap<u64, String> {
    let mut arbo: HashMap<u64, String> = HashMap::new();
    let mut inode: u64 = 2;

    arbo.insert(1, pth.to_string_lossy().to_string());

    for entry in WalkDir::new(COPIED_ROOT).into_iter().filter_map(|e| e.ok()) {
        let strpath = entry.path().display().to_string();
        if strpath != COPIED_ROOT {
            println!("indexing {}", strpath);
            arbo.insert(inode, strpath);
            inode += 1;
        } else {
            println!("ignoring {}", strpath);
        }
    }
    arbo
}

/* fn file_inode(file_name: &str) -> Result<(u64, FileType, String), Error> {
    let metadata = fs::metadata(file_name)?;
    let file_type: fuser::FileType = if (metadata.file_type().is_dir()) {
        fuser::FileType::Directory
    } else {
        fuser::FileType::RegularFile
    };
    Ok((metadata.ino(), file_type , file_name.to_owned()))
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
*/
