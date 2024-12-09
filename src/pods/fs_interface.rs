use std::{ffi::OsStr, sync::Arc};

use fuser::{FileAttr, FileType};
use tokio::sync::mpsc::UnboundedSender;

use std::{
    ffi::OsStr,
    io::{self, Write},
    os::unix::fs::FileExt,
    path::PathBuf,
};

use crate::{
    fuse::fuse_impl::TEMPLATE_FILE_ATTR, network::message::NetworkMessage, providers::FsIndex,
};

use super::{disk_manager::DiskManager, network_interface::NetworkInterface};

pub struct FsInterface {
    pub network_interface: Arc<NetworkInterface>,
}

struct Helpers {}
impl Helpers {
    pub fn wh_path_from_ino(arbo: &FsIndex, ino: &u64) -> io::Result<PathBuf> {
        match arbo.get(ino) {
            Some((_, path)) => Ok(path.to_path_buf()),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "file not found in arbo",
            )),
        }
    }

    // REVIEW discuss a better way to check this
    /// checks if a given path already exists
    /// quite costly as it loops on all path
    pub fn wh_path_exists(arbo: &FsIndex, path: &PathBuf) -> Option<u64> {
        for (ino, (_, elem_path)) in arbo.iter() {
            if *elem_path == *path {
                return Some(*ino);
            }
        }
        None
    }
}

/// Provides functions to allow primitive handlers like Fuse & WinFSP to
/// interract with the network.
impl FsInterface {
    pub fn mkfile(&self, parent_ino: u64, name: &OsStr) -> io::Result<FileAttr> {
        self.check_file_type(parent_ino, FileType::Directory)?;

        let new_path =
            Helpers::wh_path_from_ino(&self.network_interface.arbo.lock().unwrap(), &parent_ino)?
                .join(name);

        if let Some(_) =
            Helpers::wh_path_exists(&self.network_interface.arbo.lock().unwrap(), &new_path)
        {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "path already existing",
            ));
        }
        match (&self.network_interface.disk).new_file(&new_path) {
            Ok(_) => (),
            Err(e) => {
                return Err(e);
            }
        };

        // add entry to the index
        let ino = self
            .network_interface
            .register_new_file(FileType::RegularFile, new_path);

        // creating metadata to return
        let mut new_attr = TEMPLATE_FILE_ATTR;
        new_attr.ino = ino;
        new_attr.kind = FileType::RegularFile;
        new_attr.size = 0;
        Ok(new_attr)
    }

    pub fn mkdir(&mut self, parent_ino: u64, name: &OsStr) -> io::Result<FileAttr> {
        self.check_file_type(parent_ino, FileType::Directory)?;

        let new_path =
            Helpers::wh_path_from_ino(&self.network_interface.arbo.lock().unwrap(), &parent_ino)?
                .join(name);

        if let Some(_) =
            Helpers::wh_path_exists(&self.network_interface.arbo.lock().unwrap(), &new_path)
        {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "path already existing",
            ));
        }

        match (&self.network_interface.disk).new_dir(&new_path) {
            Ok(_) => (),
            Err(e) => {
                return Err(e);
            }
        };

        // adding path to the wormhole index
        let ino = self
            .network_interface
            .register_new_file(FileType::Directory, new_path);

        // creating metadata to return
        let mut new_attr = TEMPLATE_FILE_ATTR;
        new_attr.ino = ino;
        new_attr.kind = FileType::Directory;
        new_attr.size = 0;
        Ok(new_attr)
    }

    pub fn rmfile(&mut self, parent_ino: u64, name: &OsStr) -> io::Result<()> {
        let path =
            Helpers::wh_path_from_ino(&self.network_interface.arbo.lock().unwrap(), &parent_ino)?
                .join(name);

        if None == Helpers::wh_path_exists(&self.network_interface.arbo.lock().unwrap(), &path) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "file not found",
            ));
        }

        match (&self.network_interface.disk).remove_file(&path) {
            Ok(_) => (),
            Err(e) => {
                return Err(e);
            }
        };

        self.network_interface.unregister_file(path);

        Ok(())
    }

    pub fn rmdir(&mut self, parent_ino: u64, name: &OsStr) -> Option<()> {
        let _ = name;
        let _ = parent_ino;
        // should only be called on empty folders
        // if 404, not empty or file -> None
        Some(())
    }

    pub fn rename(
        &mut self,
        parent_ino: u64,
        name: &OsStr,
        newparent_ino: u64,
        newname: &OsStr,
    ) -> Option<()> {
        let _ = newname;
        let _ = newparent_ino;
        let _ = name;
        let _ = parent_ino;
        // pas clair de quand c'est appelé, si ça l'est sur des dossiers
        // non vides, go ignorer pour l'instant
        Some(())
    }

    // returns the writed size
    pub fn write(&self, ino: u64, offset: i64, data: &[u8]) -> io::Result<u32> {
        match self.index.get(&ino) {
            Some((FileType::RegularFile, _)) => {
                let path = self.mirror_path_from_inode(ino)?;
                let wrfile = self.disk.write_file(&path, S_IWRITE | S_IREAD)?;
                wrfile
                    .write_all_at(data, offset.try_into().unwrap())
                    .expect("can't write file");
                // fs::write(path, data)?;
                self.tx
                    .send(NetworkMessage::Write(ino, data.to_owned()))
                    .unwrap();
                Ok(data.len() as u32)
            }
            Some(_) => Err(io::Error::new(io::ErrorKind::Other, "File not writable")),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "File not found")),
        }
    }

    // find the path of the real file in the original folder
    pub fn mirror_path_from_inode(&self, ino: u64) -> io::Result<PathBuf> {
        if let Some(data) = self.index.get(&ino) {
            Ok(data.1.clone())
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
