use fuser::FileType;
use std::fs;
use std::{collections::HashMap, io::Error, os::unix::fs::MetadataExt, path::Path};
use walkdir::WalkDir;

use crate::fuse::start::FuseController;
use crate::pod::pod::Pod;
use crate::pod::COPIED_ROOT;

impl FuseController {
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
            Some(
                self.index
                    .clone()
                    .into_iter()
                    .filter(|e| e.1 .1.starts_with(parent_path))
                    .map(|e| e.0)
                    .collect(),
            )
        } else {
            None
        }
    }

    // returns a small amount of data for a file (asked for readdir)
    // -> (ino, type, path)
    fn file_small_meta(&self, ino: u64) -> Option<(u64, fuser::FileType, String)> {
        if let Some((file_type, file_path)) = self.index.get(&ino) {
            Some((ino, file_type.clone(), file_path.clone()))
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
