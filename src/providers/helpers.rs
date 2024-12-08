use std::{
    io,
    path::{Component, Path, PathBuf},
};

use fuser::{FileAttr, FileType};
use std::ffi::OsStr;

use crate::network::message::{FileSystemSerialized, MessageContent, ToNetworkMessage};

use super::Provider;

impl Provider {
    pub fn send_file_system(&self, origin: String) {
        let mut new_fs_index = self.index.clone();
        new_fs_index.remove(&1u64); // remove "./"
        let fs = FileSystemSerialized {
            fs_index: new_fs_index,
            next_inode: self.next_inode,
        };
        self.tx
            .send(ToNetworkMessage::SpecificMessage(
                MessageContent::FileStructure(fs),
                vec![origin],
            ))
            .expect("File system send failed");
    }

    pub fn merge_file_system(&mut self, fs: FileSystemSerialized) {
        println!("Importing other FS: own {:?} other {:?}", self.index, fs);
        for (k, v) in fs.fs_index {
            self.index.insert(k, v);
            // Handling conflicts can be implemented here
        }

        self.next_inode = fs.next_inode;
        for (_, (file_type, path)) in &self.index {
            if path.to_str().unwrap() != "./" {
                println!("Creating {:?}", path);

                match file_type {
                    fuser::FileType::NamedPipe => todo!(),
                    fuser::FileType::CharDevice => todo!(),
                    fuser::FileType::BlockDevice => todo!(),
                    fuser::FileType::Directory => self
                        .metal_handle
                        .create_dir(path, libc::S_IWRITE | libc::S_IREAD)
                        .expect("unable to create folder"),
                    fuser::FileType::RegularFile => {
                        self.metal_handle
                            .new_file(path, libc::S_IWRITE | libc::S_IREAD)
                            .expect("unable to create file");
                    }
                    fuser::FileType::Symlink => todo!(),
                    fuser::FileType::Socket => todo!(),
                };
            }
        }
        println!("Finished Mergeing file systems");
    }

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

    //STUB - copied from https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs#L61
    pub fn normalize_path(path: &Path) -> PathBuf {
        let mut components = path.components().peekable();
        let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
            components.next();
            PathBuf::from(c.as_os_str())
        } else {
            PathBuf::from("./")
        };

        for component in components {
            match component {
                Component::Prefix(..) => unreachable!(),
                Component::RootDir => {
                    ret.push(component.as_os_str());
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    ret.pop();
                }
                Component::Normal(c) => {
                    ret.push(c);
                }
            }
        }
        ret
    }
}
