use std::{ffi::OsStr, sync::Arc};

use fuser::{FileAttr, FileType};
use parking_lot::RwLock;

use std::io::{self};

use crate::{fuse::fuse_impl::TEMPLATE_FILE_ATTR, providers::whpath::WhPath};

use super::{
    disk_manager::DiskManager,
    inode::{Arbo, FsEntry, Inode, LOCK_TIMEOUT},
    network_interface::NetworkInterface,
};

pub struct FsInterface {
    pub network_interface: Arc<NetworkInterface>,
    pub disk: Arc<DiskManager>,
    pub arbo: Arc<RwLock<Arbo>>,
}

// struct Helpers {}
// impl Helpers {
// }

/// Provides functions to allow primitive handlers like Fuse & WinFSP to
/// interract with wormhole.
impl FsInterface {
    pub fn mkfile(&self, parent_ino: u64, name: String) -> io::Result<FileAttr> {
        let new_entry = FsEntry::File(vec![]);
        let new_inode: Inode = Inode::new(name, parent_ino, new_entry);
        let new_inode_id = self.network_interface.register_new_file(new_inode)?;

        let new_path: WhPath = if let Some(arbo) = self.arbo.try_read_for(LOCK_TIMEOUT) {
            arbo.get_path_from_inode_id(new_inode_id)?
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "mkfile: can't read lock arbo's RwLock",
            ));
        };

        match self
            .disk
            .new_file(&new_path)
        {
            Ok(_) => (),
            Err(e) => {
                return Err(e);
            }
        };

        // creating metadata to return
        let mut new_attr = TEMPLATE_FILE_ATTR;
        new_attr.ino = new_inode_id;
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
