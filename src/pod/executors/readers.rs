use fuser::FileType;
use std::fs;
use std::{collections::HashMap, io::Error, os::unix::fs::MetadataExt, path::Path};
use walkdir::WalkDir;

use crate::pod::pod::Pod;
use crate::pod::COPIED_ROOT;

impl Pod {
    fn file_inode(file_name: &String) -> Result<(u64, FileType, String), Error> {
        let metadata = fs::metadata(file_name)?;
        let file_type: fuser::FileType = if metadata.file_type().is_dir() {
            fuser::FileType::Directory
        } else {
            fuser::FileType::RegularFile
        };
        Ok((metadata.ino(), file_type, file_name.to_owned()))
    }

    // NOTE - dev only
    fn mirror_path_from_inode(&self, ino: u64) -> Option<&String> {
        self.index.get(&ino)
    }

    /*
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
}
