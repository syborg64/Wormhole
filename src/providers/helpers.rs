use std::{
    io,
    path::{Component, Path, PathBuf},
};

use fuser::{FileAttr, FileType};
use std::ffi::OsStr;

use super::Provider;

impl Provider {
    // find the path of the real file in the original folder
    pub fn mirror_path_from_inode(&self, ino: u64) -> io::Result<PathBuf> {
        println!("mirror path from inode");
        if let Some(data) = self.index.get(&ino) {
            println!("....>{}", data.1.display());
            Ok(data.1.clone())
        } else {
            println!("....inode NOT FOUND");

            Err(io::Error::new(io::ErrorKind::NotFound, "Inode not found"))
        }
    }

    pub fn check_file_type(&self, ino: u64, wanted_type: FileType) -> io::Result<FileAttr> {
        println!("check_file_type called");
        match self.get_metadata(ino) {
            Ok(meta) => {
                if meta.kind == wanted_type {
                    println!("file of good type");
                    Ok(meta)
                } else {
                    println!("file of wrong type");
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Specified inode not of format {:#?}", wanted_type),
                    ))
                }
            }
            Err(e) => {
                println!("get metadata FAILED");
                Err(e)
            }
        }
    }

    pub fn virt_path_from_mirror_path(&self, mirror_path: &PathBuf) -> PathBuf {
        mirror_path
            .strip_prefix(self.local_source.clone())
            .unwrap()
            .to_path_buf()
    }

    /**
     * For cases such as unlink, that gives an inode and a name
     * returns a result of (inode, FileType, name)
     */
    pub fn file_from_parent_ino_and_name(
        &self,
        parent_ino: u64,
        name: &OsStr,
    ) -> io::Result<(u64, fuser::FileType, String)> {
        match self.fs_readdir(parent_ino) {
            Ok(list) => {
                if let Some(file) = list.into_iter().find(|(_, e_type, e_name)| {
                    *e_name == name.to_string_lossy().to_string()
                        && *e_type == FileType::RegularFile
                }) {
                    Ok(file)
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "Cannot find a matching file",
                    ))
                }
            }
            Err(e) => Err(e),
        }
    }
}
