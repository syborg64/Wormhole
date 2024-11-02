use std::{io, path::PathBuf};

use fuser::{FileAttr, FileType};

use super::Provider;

impl Provider {
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

    pub fn virt_path_from_mirror_path(&self, mirror_path: PathBuf) -> String {
        mirror_path
            .strip_prefix(self.local_source.clone())
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
}
