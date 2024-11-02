use std::ops::Add;
use std::{io, path::PathBuf};

use fuser::{FileAttr, FileType};

use super::Provider;

impl Provider {
    // find the path of the real file in the original folder
    pub fn mirror_path_from_inode(&self, ino: u64) -> io::Result<String> {
        if let Some(data) = self.index.get(&ino) {
            let data = self.local_source.clone().add(&data.1);
            Ok(data)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Inode not found"))
        }
    }

    pub fn check_file_type(&self, ino: u64, wanted_type: FileType) -> io::Result<FileAttr> {
        match self.get_metadata(ino) {
            Ok(meta) => {
                if meta.kind == wanted_type {
                    Ok(meta)
                } else {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Specified inode not of format {:#?}", wanted_type),
                    ))
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn virt_path_from_mirror_path(&self, mirror_path: &PathBuf) -> String {
        mirror_path
            .strip_prefix(self.local_source.clone())
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
}
