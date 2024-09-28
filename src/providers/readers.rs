/*
 * Used for the OUTSIDE interactions
 * (actually reading mirror folder, but network one day)
 */

use fuser::FileAttr;
use log::debug;
use log::info;
use std::ffi::OsStr;
use std::fs;
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;

use super::Provider;
use super::TEMPLATE_FILE_ATTR;

// should maybe be restructured, but
// those are the functions made freely by us following our needs
// and they are directly used by the fuse lib
impl Provider {
    // Used directly in the FuseControler read function
    pub fn read(&self, ino: u64) -> Option<Vec<u8>> {
        if let Some(path) = self.mirror_path_from_inode(ino) {
            info!("mirror path from inode is {}", path);
            if let Some(content) = fs::read(Path::new(&path)).ok() {
                debug!(
                    "READ CONTENT {}",
                    String::from_utf8(content.clone()).unwrap_or("uh wtf".to_string())
                );
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
            debug!("LISTING files in parent path {:?}", parent_path);
            let test = self
                .index
                .clone()
                .into_iter()
                .filter_map(|e| {
                    if PathBuf::from(e.1 .1.clone())
                        .parent()
                        .unwrap_or(Path::new("/"))
                        == parent_path
                    {
                        Some(e.0)
                    } else {
                        None
                    }
                })
                .collect();
            debug!("LISTING RESULT {:?}", test);
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

    // used directly in FuseControler's readdir function
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

    // use real fs metadata and traduct part of it to the fuse FileAttr metadata
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
        attr.size = data.size();
        attr
    }

    // get the metadata of a file from it's inode
    pub fn get_metadata(&self, ino: u64) -> Option<FileAttr> {
        if let Some(path) = self.mirror_path_from_inode(ino) {
            info!("GET METADATA FOR PATH MIRROR {}", path);
            match fs::metadata(path) {
                Ok(data) => Some(Self::modify_metadata_template(data, ino)),
                Err(_) => None,
            }
        } else {
            None
        }
    }

    pub fn fs_lookup(&self, parent_ino: u64, file_name: &OsStr) -> Option<FileAttr> {
        let file_name = file_name.to_string_lossy().to_string();
        if let Some(datas) = self.fs_readdir(parent_ino) {
            let mut metadata: Option<FileAttr> = None;
            for data in datas {
                if data.2 == file_name {
                    metadata = self.get_metadata(data.0);
                };
            }
            metadata
        } else {
            None
        }
    }
}
