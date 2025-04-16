use crate::network::message::Address;
use crate::pods::arbo::{Arbo, FsEntry, Inode, InodeId, Metadata, LOCK_TIMEOUT};
use crate::pods::disk_manager::DiskManager;
use crate::pods::network::network_interface::{Callback, NetworkInterface};
use crate::pods::whpath::WhPath;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::io;
use std::sync::Arc;

use super::make_inode::MakeInode;

pub struct FsInterface {
    pub network_interface: Arc<NetworkInterface>,
    pub disk: DiskManager,
    pub arbo: Arc<RwLock<Arbo>>, // here only to read, as most write are made by network_interface
                                 // REVIEW - check self.arbo usage to be only reading
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum SimpleFileType {
    File,
    Directory,
}

impl Into<SimpleFileType> for &FsEntry {
    fn into(self) -> SimpleFileType {
        match self {
            FsEntry::File(_) => SimpleFileType::File,
            FsEntry::Directory(_) => SimpleFileType::Directory,
        }
    }
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
    ) -> io::Result<Inode> {
        let new_entry = match kind {
            SimpleFileType::File => FsEntry::File(vec![self.network_interface.self_addr.clone()]),
            SimpleFileType::Directory => FsEntry::Directory(Vec::new()),
        };

        let new_inode_id = match (name.as_str(), parent_ino) {
            (".global_config.toml", 1) => 2u64,
            (".local_config.toml", 1) => 3u64,
            _ => self.network_interface.get_next_inode()?,
        };

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

        match kind {
            SimpleFileType::File => {
                self.disk.new_file(new_path)?;
                ()
            }
            SimpleFileType::Directory => self.disk.new_dir(new_path)?,
        };

        Ok(new_inode)
    }

    pub fn write(&self, id: InodeId, data: &[u8], offset: u64) -> io::Result<u64> {
        let written = {
            let arbo = Arbo::read_lock(&self.arbo, "fs_interface.write")?;
            let path = arbo.get_path_from_inode_id(id)?;
            let mut meta = arbo.get_inode(id)?.meta.clone();
            drop(arbo);

            let newsize = offset + data.len() as u64;
            if newsize > meta.size {
                meta.size = newsize;
                self.network_interface.update_metadata(id, meta)?;
            }
            self.disk.write_file(path, data, offset)?
        };

        self.network_interface.revoke_remote_hosts(id)?; // TODO - manage this error to prevent remote/local desync
        Ok(written)
    }

    pub fn set_inode_meta(&self, ino: InodeId, meta: Metadata) -> io::Result<()> {
        let path = Arbo::read_lock(&self.arbo, "fs_interface::set_inode_meta")?
            .get_path_from_inode_id(ino)?;

        self.disk.set_file_size(path, meta.size)?;
        self.network_interface.update_metadata(ino, meta)
    }
    fn construct_file_path(&self, parent: InodeId, name: &String) -> io::Result<WhPath> {
        let arbo = Arbo::read_lock(&self.arbo, "fs_interface.get_begin_path_end_path")?;
        let parent_path = arbo.get_path_from_inode_id(parent)?;

        return Ok(parent_path.join(name));
    }

    // TODO:  Must handle the file creation if the file is not replicated like ino 3
    pub fn rename(
        &self,
        parent: InodeId,
        new_parent: InodeId,
        name: &String,
        new_name: &String,
    ) -> io::Result<()> {
        let parent_path = self.construct_file_path(parent, name)?;
        let new_parent_path = self.construct_file_path(new_parent, new_name)?;
        let err = self
            .disk
            .mv_file(parent_path, new_parent_path)
            .inspect_err(|err| log::error!("disk.mv_file fail: {err:?}"));
        self.network_interface
            .broadcast_rename_file(parent, new_parent, name, new_name)?;
        self.network_interface
            .arbo_rename_file(parent, new_parent, name, new_name)?;
        Ok(())
    }

    pub fn accept_rename(
        &self,
        parent: InodeId,
        new_parent: InodeId,
        name: &String,
        new_name: &String,
    ) -> io::Result<()> {
        let parent_path = self.construct_file_path(parent, name)?;
        let new_parent_path = self.construct_file_path(new_parent, new_name)?;
        if std::path::Path::new(&parent_path.inner).exists() {
            self.disk.mv_file(parent_path, new_parent_path)?;
        }
        self.network_interface
            .arbo_rename_file(parent, new_parent, name, new_name)?;
        Ok(())
    }

    // !SECTION

    // SECTION - local -> read

    pub fn get_entry_from_name(&self, parent: InodeId, name: String) -> io::Result<Inode> {
        let arbo = Arbo::read_lock(&self.arbo, "fs_interface.get_entry_from_name")?;
        arbo.get_inode_child_by_name(arbo.get_inode(parent)?, &name)
            .cloned()
    }

    pub fn read_file(&self, file: InodeId, offset: u64, len: u64) -> io::Result<Vec<u8>> {
        let cb = self.network_interface.pull_file(file)?;

        let status = match cb {
            None => true,
            Some(call) => self.network_interface.callbacks.wait_for(call)?,
        };

        if !status {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "unable to pull file",
            ));
        }

        self.disk.read_file(
            Arbo::read_lock(&self.arbo, "read_file")?.get_path_from_inode_id(file)?,
            offset,
            len,
        )
    }

    pub fn get_inode_attributes(&self, ino: InodeId) -> io::Result<Metadata> {
        let arbo = Arbo::read_lock(&self.arbo, "fs_interface::get_inode_attributes")?;

        Ok(arbo.get_inode(ino)?.meta.clone())
    }

    pub fn set_inode_attributes(&self, ino: InodeId, meta: Metadata) -> io::Result<()> {
        self.network_interface.update_metadata(ino, meta)
    }

    pub fn read_dir(&self, ino: InodeId) -> io::Result<Vec<Inode>> {
        let arbo = Arbo::read_lock(&self.arbo, "fs_interface.read_dir")?;
        let dir = arbo.get_inode(ino)?;
        //log::debug!("dir: {dir}?");
        let mut entries: Vec<Inode> = Vec::new();

        if let FsEntry::Directory(children) = &dir.entry {
            for entry in children {
                entries.push(arbo.get_inode(*entry)?.clone());
            }
            Ok(entries)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "read_dir: asked inode is not a dir",
            ))
        }
    }
    // !SECTION

    // SECTION - remote -> write

    pub fn recept_inode(&self, inode: Inode, id: InodeId) -> io::Result<()> {
        self.network_interface.acknowledge_new_file(inode, id)?;

        let new_path = {
            let arbo = Arbo::read_lock(&self.arbo, "fs_interface.write")?;
            arbo.get_path_from_inode_id(id)?
        };

        match self.disk.new_file(new_path) {
            Ok(_) => (),
            Err(e) => {
                return Err(e);
            }
        };

        Ok(())
    }

    pub fn n_recept_inode(&self, inode: Inode) -> Result<(), MakeInode> {
        self.network_interface
            .n_acknowledge_new_file(inode.clone(), inode.id)?;
        self.network_interface.n_promote_next_inode(inode.id + 1)?;

        let new_path = {
            let arbo = Arbo::n_read_lock(&self.arbo, "recept_inode")?;
            arbo.n_get_path_from_inode_id(inode.id)?
        };

        match inode.entry {
            FsEntry::File(hosts) if hosts.contains(&self.network_interface.self_addr) => self
                .disk
                .n_new_file(new_path, inode.meta.perm)
                .map(|_| ())
                .map_err(|io| MakeInode::LocalCreationFailed { io }),
            FsEntry::File(_) => Ok(()),
            FsEntry::Directory(_) => self
                .disk
                .n_new_dir(new_path, inode.meta.perm)
                .map(|_| ())
                .map_err(|io| MakeInode::LocalCreationFailed { io }),
            // TODO - remove when merge is handled because new file should create folder
            // FsEntry::Directory(_) => {}
        }
    }

    pub fn recept_binary(&self, id: InodeId, binary: Vec<u8>) -> io::Result<()> {
        let mut arbo = Arbo::write_lock(&self.arbo, "recept_binary")
            .expect("recept_binary: can't write lock arbo");
        let path = {
            match arbo.get_path_from_inode_id(id) {
                Ok(path) => path,
                Err(_) => {
                    drop(arbo);
                    return self
                        .network_interface
                        .callbacks
                        .resolve(Callback::Pull(id), false)
                        .map(|_| ());
                }
            }
        };
        let status = self.disk.write_file(path, &binary, 0).is_ok();
        self.network_interface
            .callbacks
            .resolve(Callback::Pull(id), status)?;
        if status {
            let mut hosts;
            {
                let inode = arbo.get_inode(id)?;
                if let FsEntry::File(hosts_source) = &inode.entry {
                    hosts = hosts_source.clone();
                    let self_addr = self.network_interface.self_addr.clone();
                    let idx = hosts.partition_point(|x| x <= &self_addr);
                    hosts.insert(idx, self_addr);
                } else {
                    return Err(io::ErrorKind::InvalidInput.into());
                }
            }
            arbo.set_inode_hosts(id, hosts)?;
            let inode = arbo.get_inode(id)?;
            self.network_interface.update_remote_hosts(inode)
        } else {
            Ok(())
        }
    }

    pub fn recept_remove_inode(&self, id: InodeId) -> io::Result<()> {
        let to_remove_path = {
            let arbo = Arbo::read_lock(&self.arbo, "cecept_remove_inode")?;
            arbo.get_path_from_inode_id(id)?
        };

        let _ = self.disk.remove_file(to_remove_path);

        self.network_interface.acknowledge_unregister_file(id)?;

        Ok(())
    }

    pub fn recept_edit_hosts(&self, id: InodeId, hosts: Vec<Address>) -> io::Result<()> {
        if !hosts.contains(&self.network_interface.self_addr) {
            self.disk.remove_file(
                Arbo::read_lock(&self.arbo, "recept_edit_hosts")?.get_path_from_inode_id(id)?,
            )?
        }
        self.network_interface.acknowledge_hosts_edition(id, hosts)
    }

    pub fn recept_edit_metadata(
        &self,
        id: InodeId,
        meta: Metadata,
        host: Address,
    ) -> io::Result<()> {
        self.network_interface.acknowledge_metadata(id, meta, host)
    }
    // !SECTION

    // SECTION remote -> read
    pub fn send_filesystem(&self, to: Address) -> io::Result<()> {
        self.network_interface.send_arbo(to)
    }

    pub fn register_new_node(&self, socket: Address, addr: Address) {
        self.network_interface.register_new_node(socket, addr);
    }

    pub fn send_file(&self, inode: InodeId, to: Address) -> io::Result<()> {
        let arbo = Arbo::read_lock(&self.arbo, "send_arbo")?;
        let path = arbo.get_path_from_inode_id(inode)?;
        let data = self.disk.read_file(path, 0, u64::max_value())?;
        self.network_interface.send_file(inode, data, to)
    }
}
