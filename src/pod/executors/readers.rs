use fuser::{FileAttr, FileType, ReplyAttr, Request};
use std::fs::{self, Metadata};
use std::time::{Duration, UNIX_EPOCH};
use std::{collections::HashMap, io::Error, os::unix::fs::MetadataExt, path::Path};
use walkdir::WalkDir;

use crate::pod::pod::Pod;
use crate::pod::COPIED_ROOT;

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

impl Pod {
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
