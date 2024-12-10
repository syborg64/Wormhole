use std::{ffi::OsStr, sync::Arc};

use fuser::{FileAttr, FileType};

use std::{
    io::{self},
    path::PathBuf,
};

use crate::{
    fuse::fuse_impl::TEMPLATE_FILE_ATTR,
    providers::{FsEntry, FsIndex},
};

use super::network_interface::NetworkInterface;

pub struct FsInterface {
    pub network_interface: Arc<NetworkInterface>,
}

struct Helpers {}
impl Helpers {
    pub fn entry_from_ino(arbo: &FsIndex, ino: u64) -> io::Result<FsEntry> {
        match arbo.get(&ino) {
            Some(entry) => Ok(entry.clone()),
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
        for (ino, entry) in arbo.iter() {
            if *entry.get_path() == *path {
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
        if Helpers::entry_from_ino(&self.network_interface.arbo.lock().unwrap(), parent_ino)?
            .get_filetype()
            != FileType::Directory
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path is not a dir",
            ));
        }

        let new_path =
            Helpers::entry_from_ino(&self.network_interface.arbo.lock().unwrap(), parent_ino)?
                .get_path()
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
            .register_new_file(FsEntry::File(new_path, vec![]));

        // creating metadata to return
        let mut new_attr = TEMPLATE_FILE_ATTR;
        new_attr.ino = ino;
        new_attr.kind = FileType::RegularFile;
        new_attr.size = 0;
        Ok(new_attr)
    }

    pub fn mkdir(&mut self, parent_ino: u64, name: &OsStr) -> io::Result<FileAttr> {
        if Helpers::entry_from_ino(&self.network_interface.arbo.lock().unwrap(), parent_ino)?
            .get_filetype()
            != FileType::Directory
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path is not a dir",
            ));
        }

        let new_path =
            Helpers::entry_from_ino(&self.network_interface.arbo.lock().unwrap(), parent_ino)?
                .get_path()
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
            .register_new_file(FsEntry::File(new_path, vec![]));

        // creating metadata to return
        let mut new_attr = TEMPLATE_FILE_ATTR;
        new_attr.ino = ino;
        new_attr.kind = FileType::Directory;
        new_attr.size = 0;
        Ok(new_attr)
    }
}
