use super::{
    arbo::{Arbo, FsEntry, Inode, InodeId, LOCK_TIMEOUT},
    disk_manager::DiskManager,
    network_interface::NetworkInterface,
};
use crate::providers::whpath::WhPath;
use parking_lot::RwLock;
use std::io::{self};
use std::sync::Arc;

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
    pub fn make_inode(
        &self,
        parent_ino: u64,
        name: String,
        kind: SimpleFileType,
    ) -> io::Result<(InodeId, Inode)> {
        let new_entry = match kind {
            SimpleFileType::File => FsEntry::File(Vec::new()),
            SimpleFileType::Directory => FsEntry::Directory(Vec::new()),
        };

        let new_inode: Inode = Inode::new(name, parent_ino, new_entry);
        let new_inode_id = self
            .network_interface
            .register_new_file(new_inode.clone())?;

        let new_path: WhPath = if let Some(arbo) = self.arbo.try_read_for(LOCK_TIMEOUT) {
            arbo.get_path_from_inode_id(new_inode_id)?
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "mkfile: can't read lock arbo's RwLock",
            ));
        };

        match self.disk.new_file(&new_path) {
            Ok(_) => (),
            Err(e) => {
                return Err(e);
            }
        };

        Ok((new_inode_id, new_inode))
    }

    pub fn recept_inode(&self, inode: Inode, id: InodeId) -> io::Result<()> {
        self.network_interface.acknowledge_new_file(inode, id)?;

        let new_path: WhPath = if let Some(arbo) = self.arbo.try_read_for(LOCK_TIMEOUT) {
            arbo.get_path_from_inode_id(id)?
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "mkfile: can't read lock arbo's RwLock",
            ));
        };

        match self.disk.new_file(&new_path) {
            Ok(_) => (),
            Err(e) => {
                return Err(e);
            }
        };

        Ok(())
    }

    pub fn recept_remove_inode(&self, id: InodeId) -> io::Result<()> {
        let to_remove_path: WhPath = if let Some(arbo) = self.arbo.try_read_for(LOCK_TIMEOUT) {
            arbo.get_path_from_inode_id(id)?
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "mkfile: can't read lock arbo's RwLock",
            ));
        };
        
        match self.disk.remove_file(&to_remove_path) {
            Ok(_) => (),
            Err(e) => {
                return Err(e);
            }
        };

        self.network_interface.acknowledge_unregister_file(id)?;

        Ok(())
    }
}
