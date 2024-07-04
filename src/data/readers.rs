/*
 * Used for the OUTSIDE interactions
 * (actually reading mirror folder, but network one day)
 */

use std::fs;
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};
use fuser::{FileAttr, FileType, Request};
use std::fs::Metadata;
use std::time::UNIX_EPOCH;

pub type FsIndex = HashMap<u64, (fuser::FileType, String)>;
pub struct Provider {
    pub index: FsIndex,
}

const TEMPLATE_FILE_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 13,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

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

    fn modify_metadata_template(data: Metadata, ino: u64) -> FileAttr {
        let mut attr = TEMPLATE_FILE_ATTR;
        attr.ino = ino;
        attr.kind = if data.is_dir() {
            fuser::FileType::Directory
        } else if data.is_file() {
            fuser::FileType::RegularFile
        } else {
            fuser::FileType::CharDevice // random to detect unsupported
        };
        attr
    }

    pub fn get_metadata(&self, _req: &Request, ino: u64) -> Option<FileAttr> {
        if let Some(path) = self.mirror_path_from_inode(ino) {
            match fs::metadata(path) {
                Ok(data) => Some(Self::modify_metadata_template(data, ino)),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn lookup_metadata(
        &self,
        _req: &Request,
        parent_ino: u64,
        file_name: String,
    ) -> Option<FileAttr> {
        if let Some(datas) = self.fs_readdir(parent_ino) {
            let mut metadata: Option<FileAttr> = None;
            for data in datas {
                if data.2 == file_name {
                    metadata = self.get_metadata(_req, data.0);
                };
            }
            metadata
        } else {
            None
        }
    }
}
