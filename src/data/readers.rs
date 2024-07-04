/*
 * Used for the OUTSIDE interactions
 * (actually reading mirror folder, but network one day)
 */

use std::fs;
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};

pub type FsIndex = HashMap<u64, (fuser::FileType, String)>;
pub struct Provider {
    pub index: FsIndex,
}

impl Provider {
    // NOTE - dev only
    fn mirror_path_from_inode(&self, ino: u64) -> Option<&String> {
        if let Some(data) = self.index.get(&ino) {
            Some(&data.1)
        } else {
            None
        }
    }

    pub fn read(&self, ino: u64) -> Option<Vec<u8>> {
        if let Some(path) = self.mirror_path_from_inode(ino) {
            if let Some(content) = fs::read(Path::new(&path)).ok() {
                Some(content)
            } else {
                None
            }
        } else {
            None
        }
    }

    // list files inodes in the parent folder
    fn list_files(&self, parent_ino: u64) -> Option<Vec<u64>> {
        if let Some((_, parent_path)) = self.index.get(&parent_ino) {
            let parent_path = Path::new(&parent_path);
            println!("LISTING files in parent path {:?}", parent_path);
            let test = self
                .index
                .clone()
                .into_iter()
                .map(|(a, (b, c))| (a, (b, PathBuf::from(c))))
                .filter(|e| e.1 .1.parent().unwrap() == parent_path)
                .map(|e| e.0)
                .collect();
            println!("LISTING RESULT {:?}", test);
            Some(test)
        } else {
            None
        }
    }

    // returns a small amount of data for a file (asked for readdir)
    // -> (ino, type, name)
    fn file_small_meta(&self, ino: u64) -> Option<(u64, fuser::FileType, String)> {
        if let Some((file_type, file_path)) = self.index.get(&ino) {
            let file_name = if let Some(name) = Path::new(file_path).file_name() {
                name.to_string_lossy().to_string()
            } else {
                "errorname".to_string()
            };
            Some((ino, file_type.clone(), file_name))
        } else {
            None
        }
    }

    pub fn fs_readdir(&self, parent_ino: u64) -> Option<Vec<(u64, fuser::FileType, String)>> {
        if let Some(list) = self.list_files(parent_ino) {
            Some(
                list.into_iter()
                    .filter_map(|e| self.file_small_meta(e))
                    .collect(),
            )
        } else {
            None
        }
    }
}
