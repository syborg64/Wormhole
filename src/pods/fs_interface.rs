use super::whpath::WhPath;
use super::{
    arbo::{Arbo, FsEntry, Inode, InodeId, LOCK_TIMEOUT},
    disk_manager::DiskManager,
    network_interface::NetworkInterface,
};
use parking_lot::RwLock;
use std::io::{self};
use std::sync::Arc;

pub struct FsInterface {
    pub network_interface: Arc<NetworkInterface>,
    pub disk: DiskManager,
    pub arbo: Arc<RwLock<Arbo>>, // here only to read, as most write are made by network_interface
}

pub enum SimpleFileType {
    File,
    Directory,
}

/// Provides functions to allow primitive handlers like Fuse & WinFSP to
/// interract with wormhole.
impl FsInterface {
    pub fn new(
        network_interface: Arc<NetworkInterface>,
        disk_manager: DiskManager,
        arbo: Arc<RwLock<Arbo>>,
    ) -> Self {
        Self {
            network_interface,
            disk: disk_manager,
            arbo,
        }
    }

    // SECTION - local -> write

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

        let new_inode_id = self.network_interface.get_next_inode()?;
        let new_inode: Inode = Inode::new(name, parent_ino, new_inode_id, new_entry);
        self.network_interface
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

    // !SECTION

    // SECTION - local -> read

    pub fn get_entry_from_name(&self, parent: InodeId, name: String) -> io::Result<Inode> {
        if let Some(arbo) = self.arbo.try_read_for(LOCK_TIMEOUT) {
            arbo.get_inode_child_by_name(arbo.get_inode(parent)?, &name)
                .cloned()
        } else {
            Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "lookup: can't read lock arbo's RwLock",
            ))
        }
    }

    pub fn read_file(&self, file: InodeId, offset: u64, len: u64) {}
    // !SECTION

    // SECTION - remote -> write
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

    pub fn recept_binary(&self, id: InodeId) -> io::Result<()> {
        let path = Arbo::read_lock(&self.arbo, "coucou")?.get_path_from_inode_id(id)?;
        Ok(())
    }

    pub fn remove_inode(&self, id: InodeId) -> io::Result<()> {
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

        self.network_interface.unregister_file(id)?;

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

    // !SECTION

    // SECTION - Adapters
    // NOTE - system specific (fuse/winfsp) code that need access to arbo or other classes

    pub fn fuse_remove_inode(&self, parent: InodeId, name: &std::ffi::OsStr) -> io::Result<()> {
        let target = if let Some(arbo) = self.arbo.try_read_for(LOCK_TIMEOUT) {
            let parent = arbo.get_inode(parent)?;
            arbo.get_inode_child_by_name(parent, &name.to_string_lossy().to_string())?
                .id
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "mkfile: can't read lock arbo's RwLock",
            ));
        };
        self.remove_inode(target)
    }

    // !SECTION
}
