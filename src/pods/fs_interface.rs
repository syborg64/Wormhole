use std::{ffi::OsStr, sync::Arc};

use fuser::{FileAttr, FileType};
use parking_lot::RwLock;

use std::io::{self};

use crate::{fuse::fuse_impl::TEMPLATE_FILE_ATTR, providers::whpath::WhPath};

use super::{
    disk_manager::DiskManager,
    inode::{Arbo, FsEntry, Inode, InodeId, LOCK_TIMEOUT},
    network_interface::NetworkInterface,
};

pub struct FsInterface {
    pub network_interface: Arc<NetworkInterface>,
    pub disk: Arc<DiskManager>,
    pub arbo: Arc<RwLock<Arbo>>,
}

pub enum SimpleFileType {
    File,
    Directory,
}

/// Provides functions to allow primitive handlers like Fuse & WinFSP to
/// interract with wormhole.
impl FsInterface {
    pub fn mknod(&self, parent_ino: u64, name: String, kind: SimpleFileType) -> io::Result<(InodeId, Inode)> {
        let new_entry = match kind {
            SimpleFileType::File => FsEntry::File(Vec::new()),
            SimpleFileType::Directory => FsEntry::Directory(Vec::new()),
        };

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

        // // creating metadata to return
        // let mut new_attr = TEMPLATE_FILE_ATTR;
        // new_attr.ino = new_inode_id;
        // new_attr.kind = match kind {
        //     SimpleFileType::File => FileType::RegularFile,
        //     SimpleFileType::Directory => FileType::Directory,
        // };
        // new_attr.size = 0;
        // Ok(new_attr)

        Ok((new_inode_id, new_inode))
    }
}
