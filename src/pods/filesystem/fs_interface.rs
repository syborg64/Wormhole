use crate::error::WhResult;
use crate::network::message::Address;
use crate::pods::arbo::{Arbo, FsEntry, Inode, InodeId, Metadata};
use crate::pods::disk_manager::DiskManager;
use crate::pods::network::callbacks::Callback;
use crate::pods::network::network_interface::NetworkInterface;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::io;
use std::sync::Arc;

use super::file_handle::FileHandleManager;
use super::make_inode::MakeInodeError;

pub struct FsInterface {
    pub network_interface: Arc<NetworkInterface>,
    pub disk: Box<dyn DiskManager>,
    pub file_handles: Arc<RwLock<FileHandleManager>>,
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
        disk_manager: Box<dyn DiskManager>,
        arbo: Arc<RwLock<Arbo>>,
    ) -> Self {
        Self {
            network_interface,
            disk: disk_manager,
            file_handles: Arc::new(RwLock::new(FileHandleManager::new())),
            arbo,
        }
    }

    // SECTION - local -> write
    pub fn set_inode_meta(&self, ino: InodeId, meta: Metadata) -> io::Result<()> {
        let path = Arbo::read_lock(&self.arbo, "fs_interface::set_inode_meta")?
            .get_path_from_inode_id(ino)?;

        self.disk.set_file_size(&path, meta.size as usize)?;
        self.network_interface.update_metadata(ino, meta)
    }


    // !SECTION

    // SECTION - local -> read

    pub fn get_entry_from_name(&self, parent: InodeId, name: String) -> io::Result<Inode> {
        let arbo = Arbo::read_lock(&self.arbo, "fs_interface.get_entry_from_name")?;
        arbo.get_inode_child_by_name(arbo.get_inode(parent)?, &name)
            .cloned()
    }

    // NOTE - ignores the callback. To pull a file normaly, please use a process similar to read_file
    pub async fn pull_file_async(&self, file: InodeId) -> io::Result<()> {
        self.network_interface
            .pull_file_async(file)
            .await
            .map(|_| ())
    }

    pub fn get_inode_attributes(&self, ino: InodeId) -> io::Result<Metadata> {
        let arbo = Arbo::read_lock(&self.arbo, "fs_interface::get_inode_attributes")?;

        Ok(arbo.get_inode(ino)?.meta.clone())
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
    pub fn recept_inode(&self, inode: Inode) -> Result<(), MakeInodeError> {
        self.network_interface
            .acknowledge_new_file(inode.clone(), inode.id)?;
        self.network_interface.promote_next_inode(inode.id + 1)?;

        let new_path = {
            let arbo = Arbo::n_read_lock(&self.arbo, "recept_inode")?;
            arbo.n_get_path_from_inode_id(inode.id)?
        };

        match inode.entry {
            FsEntry::File(hosts) if hosts.contains(&self.network_interface.self_addr) => self
                .disk
                .new_file(&new_path, inode.meta.perm)
                .map(|_| ())
                .map_err(|io| MakeInodeError::LocalCreationFailed { io }),
            FsEntry::File(_) => Ok(()),
            FsEntry::Directory(_) => self
                .disk
                .new_dir(&new_path, inode.meta.perm)
                .map(|_| ())
                .map_err(|io| MakeInodeError::LocalCreationFailed { io }),
            // TODO - remove when merge is handled because new file should create folder
            // FsEntry::Directory(_) => {}
        }
    }

    pub fn recept_binary(&self, id: InodeId, binary: Vec<u8>) -> io::Result<()> {
        let (path, inode) = {
            let arbo = Arbo::read_lock(&self.arbo, "recept_binary")
                .expect("recept_binary: can't read lock arbo");
            (
                match arbo.get_path_from_inode_id(id) {
                    Ok(path) => path,
                    Err(_) => {
                        return self
                            .network_interface
                            .callbacks
                            .resolve(Callback::Pull(id), false)
                    }
                },
                arbo.get_inode(id)?.clone(),
            )
        };

        let _created = self.disk.new_file(&path, inode.meta.perm);
        let status = self
            .disk
            .write_file(&path, &binary, 0)
            .inspect_err(|e| log::error!("writing pulled file: {e}"));
        let _ = self
            .network_interface
            .callbacks
            .resolve(Callback::Pull(id), status.is_ok());
        status?;
        let mut hosts;
        if let FsEntry::File(hosts_source) = &inode.entry {
            hosts = hosts_source.clone();
            let self_addr = self.network_interface.self_addr.clone();
            let idx = hosts.partition_point(|x| x <= &self_addr);
            hosts.insert(idx, self_addr);
        } else {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        Arbo::write_lock(&self.arbo, "recept_binary")
            .expect("recept_binary: can't write lock arbo")
            .set_inode_hosts(id, hosts)?;
        self.network_interface.update_remote_hosts(&inode)
    }

    pub fn recept_edit_hosts(&self, id: InodeId, hosts: Vec<Address>) -> WhResult<()> {
        if !hosts.contains(&self.network_interface.self_addr) {
            let path =
                Arbo::n_read_lock(&self.arbo, "recept_edit_hosts")?.n_get_path_from_inode_id(id)?;
            if let Err(e) = self.disk.remove_file(&path) {
                log::error!("recept_edit_hosts: can't delete file. {}", e);
            }
        }
        self.network_interface.acknowledge_hosts_edition(id, hosts)
    }

    pub fn recept_revoke_hosts(&self, id: InodeId, host: Address, meta: Metadata) -> WhResult<()> {
        if host != self.network_interface.self_addr {
            // TODO: recept_revoke_hosts, for the redudancy, should recieve the written text (data from write) instead of deleting and adding it back completely with apply_redudancy
            if let Err(e) = self.disk.remove_file(
                &Arbo::n_read_lock(&self.arbo, "recept_revoke_hosts")?
                    .n_get_path_from_inode_id(id)?,
            ) {
                log::debug!("recept_revoke_hosts: can't delete file. {}", e);
            }
        }
        self.network_interface.acknowledge_metadata(id, meta)?;
        self.network_interface
            .acknowledge_hosts_edition(id, vec![host])
    }

    pub fn recept_add_hosts(&self, id: InodeId, hosts: Vec<Address>) -> io::Result<()> {
        self.network_interface.aknowledge_new_hosts(id, hosts)
    }

    pub fn recept_remove_hosts(&self, id: InodeId, hosts: Vec<Address>) -> io::Result<()> {
        if hosts.contains(&self.network_interface.self_addr) {
            let path =
                Arbo::read_lock(&self.arbo, "recept_remove_hosts")?.get_path_from_inode_id(id)?;
            if let Err(e) = self.disk.remove_file(&path) {
                log::debug!("recept_remove_hosts: can't delete file. {}", e);
            }
        }

        self.network_interface.aknowledge_hosts_removal(id, hosts)
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
        let mut size = arbo.get_inode(inode)?.meta.size as usize;
        let mut data = Vec::new();
        data.resize(size, 0);
        size = self.disk.read_file(&path, 0, &mut data)?;
        data.resize(size, 0);
        self.network_interface.send_file(inode, data, to)
    }
}
