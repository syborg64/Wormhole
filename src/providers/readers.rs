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
use std::io;
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
    pub fn read(&self, ino: u64) -> io::Result<Vec<u8>> {
        println!("read called on ino {}", ino); // DEBUG
        match self.mirror_path_from_inode(ino) {
            Ok(path) => {
                info!("mirror path from inode is {:?}", path);
                fs::read(Path::new(&path))
            }
            Err(e) => Err(e),
        }
    }

    // list files inodes in the parent folder
    // List from hashmap and not from disk
    fn list_files(&self, parent_ino: u64) -> io::Result<Vec<u64>> {
        println!("list files called on ino {}", parent_ino); // DEBUG
        match self.index.get(&parent_ino) {
            Some((_, parent_path)) => {
                debug!("LISTING files in parent path {:?}", parent_path);
                let ino_list = self
                    .index
                    .iter()
                    .filter_map(|e| {
                        let tested_path = &e.1 .1;
                        println!("...Tested path {}", tested_path.display());
                        let tested_parent_path = tested_path.parent().unwrap_or(Path::new("/"));
                        println!("...Parent path {}", tested_parent_path.display());
                        if tested_parent_path == parent_path {
                            Some(e.0.clone())
                        } else {
                            println!(
                                "bahahahahahah none on path ({}) & parent ({})",
                                tested_path.display(),
                                tested_parent_path.display()
                            );
                            None
                        }
                    })
                    .collect();
                debug!("LISTING RESULT {:?}", ino_list);
                Ok(ino_list)
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Parent inode not found",
            )),
        }
    }

    // returns a small amount of data for a file (asked for readdir)
    // -> (ino, type, name)
    fn file_small_meta(&self, ino: u64) -> io::Result<(u64, fuser::FileType, String)> {
        println!("file_small_meta called on ino {}", ino); // DEBUG
        match self.index.get(&ino) {
            Some((file_type, file_path)) => {
                let file_name = match Path::new(file_path).file_name() {
                    Some(name) => name.to_string_lossy().to_string(),
                    None => {
                        return Err(io::Error::new(io::ErrorKind::Other, "Invalid path ending"))
                    }
                };
                Ok((ino, file_type.clone(), file_name))
            }
            None => Err(io::Error::new(io::ErrorKind::NotFound, "Inode not found")),
        }
    }

    // used directly in FuseControler's readdir function
    pub fn fs_readdir(&self, parent_ino: u64) -> io::Result<Vec<(u64, fuser::FileType, String)>> {
        println!("fs_readdir called on ino {}", parent_ino); // DEBUG
        match self.list_files(parent_ino) {
            Ok(list) => Ok(list
                .into_iter()
                .filter_map(|e| self.file_small_meta(e).ok())
                .collect()),
            Err(e) => Err(e),
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
    pub fn get_metadata(&self, ino: u64) -> io::Result<FileAttr> {
        println!("get_metadata called on ino {}", ino); // DEBUG
        match self.mirror_path_from_inode(ino) {
            Ok(path) => {
                debug!("GET METADATA FOR PATH MIRROR {:?}", path);
                match fs::metadata(path) {
                    Ok(data) => Ok(Self::modify_metadata_template(data, ino)),
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn fs_lookup(&self, parent_ino: u64, file_name: &OsStr) -> io::Result<FileAttr> {
        println!(
            "fs_lookup called on ino {} and file name {:#?}",
            parent_ino, file_name
        ); // DEBUG
        let file_name = file_name.to_string_lossy().to_string();
        match self.fs_readdir(parent_ino) {
            Ok(datas) => {
                let mut metadata: io::Result<FileAttr> =
                    Err(io::Error::new(io::ErrorKind::NotFound, "Path not found"));
                for data in datas {
                    if data.2 == file_name {
                        metadata = self.get_metadata(data.0);
                    };
                }
                metadata
            }
            Err(e) => Err(e),
        }
    }
}
