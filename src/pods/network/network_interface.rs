use std::{
    collections::HashMap,
    io::{self, ErrorKind},
    sync::Arc,
};

use crate::{
    error::{WhError, WhResult},
    network::message::{MessageAndStatus, RedundancyMessage},
};
use parking_lot::{Mutex, RwLock};
use tokio::sync::{
    broadcast,
    mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};

use crate::network::{
    message::{
        self, Address, FileSystemSerialized, FromNetworkMessage, MessageContent, ToNetworkMessage,
    },
    peer_ipc::PeerIPC,
    server::Server,
};
use crate::pods::{
    arbo::{FsEntry, Metadata},
    filesystem::{make_inode::MakeInode, remove_inode::RemoveInode},
    whpath::WhPath,
};

use crate::pods::{
    arbo::{Arbo, Inode, InodeId, LOCK_TIMEOUT},
    filesystem::fs_interface::FsInterface,
};

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub enum Callback {
    Pull(InodeId),
    PullFs,
}

#[derive(Debug)]
pub struct Callbacks {
    callbacks: RwLock<HashMap<Callback, broadcast::Sender<bool>>>,
}

impl Callbacks {
    pub fn create(&self, call: Callback) -> io::Result<Callback> {
        if let Some(mut callbacks) = self.callbacks.try_write_for(LOCK_TIMEOUT) {
            if !callbacks.contains_key(&call) {
                let (tx, _) = broadcast::channel(1);

                callbacks.insert(call, tx);
            };
            Ok(call)
        } else {
            Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "unable to write_lock callbacks",
            ))
        }
    }

    pub fn resolve(&self, call: Callback, status: bool) -> io::Result<()> {
        if let Some(mut callbacks) = self.callbacks.try_write_for(LOCK_TIMEOUT) {
            if let Some(cb) = callbacks.remove(&call) {
                cb.send(status).map(|_| ()).map_err(|send_error| {
                    io::Error::new(io::ErrorKind::AddrNotAvailable, send_error.to_string())
                })
            } else {
                Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "no such callback active",
                ))
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "unable to read_lock callbacks",
            ))
        }
    }

    pub fn wait_for(&self, call: Callback) -> io::Result<bool> {
        let mut waiter = if let Some(callbacks) = self.callbacks.try_read_for(LOCK_TIMEOUT) {
            if let Some(cb) = callbacks.get(&call) {
                cb.subscribe()
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "no such callback active",
                ));
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "unable to read_lock callbacks",
            ));
        };

        match waiter.blocking_recv() {
            Ok(status) => Ok(status),
            Err(_) => Ok(false), // maybe change to a better handling
        }
    }

    pub async fn async_wait_for(&self, call: Callback) -> io::Result<bool> {
        let mut waiter = if let Some(callbacks) = self.callbacks.try_read_for(LOCK_TIMEOUT) {
            if let Some(cb) = callbacks.get(&call) {
                cb.subscribe()
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "no such callback active",
                ));
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "unable to read_lock callbacks",
            ));
        };

        match waiter.recv().await {
            Ok(status) => Ok(status),
            Err(_) => Ok(false), // maybe change to a better handling
        }
    }
}

pub fn get_all_peers_address(peers: &Arc<RwLock<Vec<PeerIPC>>>) -> WhResult<Vec<Address>> {
    Ok(peers
        .try_read_for(LOCK_TIMEOUT)
        .ok_or(WhError::WouldBlock {
            called_from: "apply_redundancy: can't lock peers mutex".to_string(),
        })?
        .iter()
        .map(|peer| peer.address.clone())
        .collect::<Vec<Address>>())
}
#[derive(Debug)]
pub struct NetworkInterface {
    pub arbo: Arc<RwLock<Arbo>>,
    pub mount_point: WhPath,
    pub to_network_message_tx: UnboundedSender<ToNetworkMessage>,
    pub to_redundancy_tx: UnboundedSender<RedundancyMessage>,
    pub next_inode: Mutex<InodeId>, // TODO - replace with InodeIndex type
    pub callbacks: Callbacks,
    pub peers: Arc<RwLock<Vec<PeerIPC>>>,
    pub self_addr: Address,
    pub redundancy: u64,
}

impl NetworkInterface {
    pub fn new(
        arbo: Arc<RwLock<Arbo>>,
        mount_point: WhPath,
        to_network_message_tx: UnboundedSender<ToNetworkMessage>,
        to_redundancy_tx: UnboundedSender<RedundancyMessage>,
        next_inode: InodeId,
        peers: Arc<RwLock<Vec<PeerIPC>>>,
        self_addr: Address,
        redundancy: u64,
    ) -> Self {
        let next_inode = Mutex::new(next_inode);

        Self {
            arbo,
            mount_point,
            to_network_message_tx,
            to_redundancy_tx,
            next_inode,
            callbacks: Callbacks {
                callbacks: HashMap::new().into(),
            },
            peers,
            self_addr,
            redundancy,
        }
    }

    pub fn get_next_inode(&self) -> io::Result<u64> {
        let mut next_inode = match self.next_inode.try_lock_for(LOCK_TIMEOUT) {
            Some(lock) => Ok(lock),
            None => Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "get_next_inode: can't lock next_inode",
            )),
        }?;
        let available_inode = *next_inode;
        *next_inode += 1;

        Ok(available_inode)
    }

    /** TODO: Doc when reviews are finished */
    pub fn n_get_next_inode(&self) -> WhResult<u64> {
        let mut next_inode =
            self.next_inode
                .try_lock_for(LOCK_TIMEOUT)
                .ok_or(WhError::WouldBlock {
                    called_from: "get_next_inode".to_string(),
                })?;

        let available_inode = *next_inode;
        *next_inode += 1;

        Ok(available_inode)
    }

    #[must_use]
    pub fn promote_next_inode(&self, new: u64) -> WhResult<()> {
        let mut next_inode =
            self.next_inode
                .try_lock_for(LOCK_TIMEOUT)
                .ok_or(WhError::WouldBlock {
                    called_from: "promote_next_inode".to_string(),
                })?;

        // REVIEW: next_inode being behind a mutex is weird and
        // the function not taking a mutable ref feels weird, is next_inode behind a mutex just to allow a simple &self?
        if *next_inode < new {
            *next_inode = new;
        };
        Ok(())
    }

    #[must_use]
    /// Add the requested entry to the arbo and inform the network
    pub fn register_new_inode(&self, inode: Inode) -> Result<(), MakeInode> {
        let inode_id = inode.id.clone();
        Arbo::n_write_lock(&self.arbo, "register_new_inode")?.n_add_inode(inode.clone())?;

        if inode_id != 3u64 {
            self.to_network_message_tx
                .send(ToNetworkMessage::BroadcastMessage(
                    message::MessageContent::Inode(inode),
                ))
                .expect("register inode: unable to update modification on the network thread");
        }
        Ok(())
        // TODO - if unable to update for some reason, should be passed to the background worker
    }

    pub fn broadcast_rename_file(
        &self,
        parent: InodeId,
        new_parent: InodeId,
        name: &String,
        new_name: &String,
    ) -> io::Result<()> {
        self.to_network_message_tx
            .send(ToNetworkMessage::BroadcastMessage(
                message::MessageContent::Rename(parent, new_parent, name.clone(), new_name.clone()),
            ))
            .expect("broadcast_rename_file: unable to update modification on the network thread");
        Ok(())
    }

    pub fn arbo_rename_file(
        &self,
        parent: InodeId,
        new_parent: InodeId,
        name: &String,
        new_name: &String,
    ) -> io::Result<()> {
        let mut arbo = Arbo::write_lock(&self.arbo, "arbo_rename_file")?;

        arbo.mv_inode(parent, new_parent, name, new_name)
    }

    #[must_use]
    /// Get a new inode, add the requested entry to the arbo and inform the network
    pub fn acknowledge_new_file(&self, inode: Inode, _id: InodeId) -> Result<(), MakeInode> {
        let mut arbo = Arbo::n_write_lock(&self.arbo, "acknowledge_new_file")?;
        arbo.n_add_inode(inode)
    }

    /// Remove [Inode] from the [Arbo] and inform the network of the removal
    pub fn unregister_inode(&self, id: InodeId) -> Result<(), RemoveInode> {
        Arbo::n_write_lock(&self.arbo, "unregister_inode")?.n_remove_inode(id)?;

        if id != 3u64 {
            self.to_network_message_tx
                .send(ToNetworkMessage::BroadcastMessage(
                    message::MessageContent::Remove(id),
                ))
                .expect("unregister_inode: unable to update modification on the network thread");
        }
        // TODO - if unable to update for some reason, should be passed to the background worker
        Ok(())
    }

    /// Remove [Inode] from the [Arbo]
    pub fn acknowledge_unregister_inode(&self, id: InodeId) -> Result<Inode, RemoveInode> {
        Arbo::n_write_lock(&self.arbo, "acknowledge_unregister_inode")?.n_remove_inode(id)
    }

    pub fn acknowledge_hosts_edition(&self, id: InodeId, hosts: Vec<Address>) -> io::Result<()> {
        let mut arbo = Arbo::write_lock(&self.arbo, "acknowledge_hosts_edition")?;

        arbo.set_inode_hosts(id, hosts) // TODO - if unable to update for some reason, should be passed to the background worker
    }

    pub fn acknowledge_metadata(
        &self,
        id: InodeId,
        meta: Metadata,
        host: Address,
    ) -> io::Result<()> {
        let mut arbo = Arbo::write_lock(&self.arbo, "acknowledge_metadata")?;
        arbo.set_inode_hosts(id, vec![host])?;
        arbo.set_inode_meta(id, meta) // TODO - if unable to update for some reason, should be passed to the background worker
    }

    // REVIEW - recheck and simplify this if possible
    pub async fn pull_file_async(&self, file: InodeId) -> io::Result<Option<Callback>> {
        let hosts = {
            let arbo = Arbo::read_lock(&self.arbo, "pull_file")?;
            if let FsEntry::File(hosts) = &arbo.get_inode(file)?.entry {
                hosts.clone()
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "pull_file: can't pull a folder",
                ));
            }
        };

        if hosts.len() == 0 {
            log::error!("No hosts hold the file");
            return Err(io::ErrorKind::InvalidData.into());
        }

        if hosts.contains(&self.self_addr) {
            // if the asked file is already on disk
            Ok(None)
        } else {
            let callback = self.callbacks.create(Callback::Pull(file))?;
            let (status_tx, mut status_rx) = tokio::sync::mpsc::unbounded_channel::<WhResult<()>>();

            // will try to pull on all redundancies until success
            for host in hosts {
                // trying on host `pull_from`
                self.to_network_message_tx
                    .send(ToNetworkMessage::SpecificMessage(
                        (
                            MessageContent::RequestFile(file, self.self_addr.clone()),
                            Some(status_tx.clone()),
                        ),
                        vec![host.clone()], // NOTE - naive choice for now
                    ))
                    .expect("pull_file: unable to request on the network thread");

                // processing status
                match status_rx
                    .recv()
                    .await
                    .expect("pull_file: unable to get status from the network thread")
                {
                    Ok(()) => return Ok(Some(callback)),
                    Err(_) => continue,
                }
            }
            let _ = self.callbacks.resolve(callback, true);
            log::error!("No host is currently able to send the file\nFile: {file}");
            return Err(io::ErrorKind::NotConnected.into());
        }
    }

    // REVIEW - recheck and simplify this if possible
    pub fn pull_file_sync(&self, file: InodeId) -> io::Result<Option<Callback>> {
        let hosts = {
            let arbo = Arbo::read_lock(&self.arbo, "pull_file")?;
            if let FsEntry::File(hosts) = &arbo.get_inode(file)?.entry {
                hosts.clone()
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "pull_file: can't pull a folder",
                ));
            }
        };

        if hosts.len() == 0 {
            log::error!("No hosts hold the file");
            return Err(io::ErrorKind::InvalidData.into());
        }

        if hosts.contains(&self.self_addr) {
            // if the asked file is already on disk
            Ok(None)
        } else {
            let callback = self.callbacks.create(Callback::Pull(file))?;
            let (status_tx, mut status_rx) = tokio::sync::mpsc::unbounded_channel::<WhResult<()>>();

            // will try to pull on all redundancies until success
            for host in hosts {
                // trying on host `pull_from`
                self.to_network_message_tx
                    .send(ToNetworkMessage::SpecificMessage(
                        (
                            MessageContent::RequestFile(file, self.self_addr.clone()),
                            Some(status_tx.clone()),
                        ),
                        vec![host.clone()], // NOTE - naive choice for now
                    ))
                    .expect("pull_file: unable to request on the network thread");

                // processing status
                match status_rx
                    .blocking_recv()
                    .expect("pull_file: unable to get status from the network thread")
                {
                    Ok(()) => return Ok(Some(callback)),
                    Err(_) => continue,
                }
            }
            let _ = self.callbacks.resolve(callback, true);
            log::error!("No host is currently able to send the file\nFile: {file}");
            return Err(io::ErrorKind::NotConnected.into());
        }
    }

    pub fn send_file(&self, inode: InodeId, data: Vec<u8>, to: Address) -> io::Result<()> {
        self.to_network_message_tx
            .send(ToNetworkMessage::SpecificMessage(
                (MessageContent::PullAnswer(inode, data), None),
                vec![to],
            ))
            .expect("send_file: unable to update modification on the network thread");
        Ok(())
    }

    pub async fn send_file_redundancy(
        &self,
        inode: InodeId,
        data: Vec<u8>,
        to: Address,
    ) -> WhResult<Address> {
        let (status_tx, mut status_rx) = unbounded_channel();
        self.to_network_message_tx
            .send(ToNetworkMessage::SpecificMessage(
                (MessageContent::RedundancyFile(inode, data), Some(status_tx)),
                vec![to.clone()],
            ))
            .expect("send_file: unable to update modification on the network thread");
        status_rx
            .recv()
            .await
            .unwrap_or(Err(WhError::NetworkDied {
                called_from: "network_interface::send_file_redundancy".to_owned(),
            }))
            .map(|()| to)
    }

    pub fn revoke_remote_hosts(&self, id: InodeId) -> WhResult<()> {
        self.update_hosts(id, vec![self.self_addr.clone()])?;
        self.apply_redundancy(id);
        Ok(())
    }

    pub fn add_inode_hosts(&self, ino: InodeId, hosts: Vec<Address>) -> WhResult<()> {
        Arbo::n_write_lock(&self.arbo, "network_interface::update_hosts")?
            .n_add_inode_hosts(ino, hosts)?;
        self.update_remote_hosts(ino)
    }

    pub fn update_hosts(&self, ino: InodeId, hosts: Vec<Address>) -> WhResult<()> {
        Arbo::n_write_lock(&self.arbo, "network_interface::update_hosts")?
            .n_set_inode_hosts(ino, hosts)?;
        self.update_remote_hosts(ino)
    }

    fn update_remote_hosts(&self, ino: InodeId) -> WhResult<()> {
        let inode = Arbo::n_read_lock(&self.arbo, "update_remote_hosts")?
            .n_get_inode(ino)?
            .clone();

        if let FsEntry::File(hosts) = &inode.entry {
            self.to_network_message_tx
                .send(ToNetworkMessage::BroadcastMessage(
                    MessageContent::EditHosts(inode.id, hosts.clone()),
                ))
                .expect("update_remote_hosts: unable to update modification on the network thread");
            Ok(())
        } else {
            Err(WhError::InodeIsADirectory {
                detail: "update_remote_hosts".to_owned(),
            })
        }
    }

    // pub fn update_remote_hosts(&self, inode: &Inode) -> io::Result<()> {
    //     if let FsEntry::File(hosts) = &inode.entry {
    //         self.to_network_message_tx
    //             .send(ToNetworkMessage::BroadcastMessage(
    //                 MessageContent::EditHosts(inode.id, hosts.clone()),
    //             ))
    //             .expect("update_remote_hosts: unable to update modification on the network thread");
    //         Ok(())
    //     } else {
    //         Err(io::ErrorKind::InvalidInput.into())
    //     }
    // }

    pub fn aknowledge_new_hosts(&self, id: InodeId, new_hosts: Vec<Address>) -> io::Result<()> {
        Arbo::write_lock(&self.arbo, "aknowledge_new_hosts")?.add_inode_hosts(id, new_hosts)
    }

    pub fn aknowledge_hosts_removal(&self, id: InodeId, new_hosts: Vec<Address>) -> io::Result<()> {
        Arbo::write_lock(&self.arbo, "aknowledge_hosts_removal")?.remove_inode_hosts(id, new_hosts)
    }

    pub fn update_metadata(&self, id: InodeId, meta: Metadata) -> io::Result<()> {
        let mut arbo = Arbo::write_lock(&self.arbo, "network_interface::update_metadata")?;
        arbo.set_inode_meta(id, meta.clone())?;

        self.to_network_message_tx
            .send(ToNetworkMessage::BroadcastMessage(
                MessageContent::EditMetadata(id, meta, self.self_addr.clone()),
            ))
            .expect("update_metadata: unable to update modification on the network thread");
        Ok(())
        /* REVIEW
         * This system (and others broadcasts systems) should be reviewed as they don't check success.
         * In this case, if another host misses this order, it will not update it's file.
         * We could create a "broadcast" callback with the number of awaited confirmations and a timeout
         * before resend or fail declaration.
         * Or send a bunch of Specific messages
         */
    }

    pub fn n_update_metadata(&self, id: InodeId, meta: Metadata) -> WhResult<()> {
        let mut arbo = Arbo::n_write_lock(&self.arbo, "network_interface::update_metadata")?;
        arbo.n_set_inode_meta(id, meta.clone())?;

        self.to_network_message_tx
            .send(ToNetworkMessage::BroadcastMessage(
                MessageContent::EditMetadata(id, meta, self.self_addr.clone()),
            ))
            .expect("update_metadata: unable to update modification on the network thread");
        Ok(())
        /* REVIEW
         * This system (and others broadcasts systems) should be reviewed as they don't check success.
         * In this case, if another host misses this order, it will not update it's file.
         * We could create a "broadcast" callback with the number of awaited confirmations and a timeout
         * before resend or fail declaration.
         * Or send a bunch of Specific messages
         */
    }

    // SECTION Redundancy related

    pub fn apply_redundancy(&self, file_id: InodeId) {
        self.to_redundancy_tx
            .send(RedundancyMessage::ApplyTo(file_id))
            .expect("network_interface::apply_redundancy: tx error");
    }

    // !SECTION ^ Redundancy related

    // SECTION Node related

    pub fn register_to_others(&self) {
        self.to_network_message_tx
            .send(ToNetworkMessage::BroadcastMessage(
                MessageContent::Register(self.self_addr.clone()),
            ))
            .expect("register_to_others: unable to update modification on the network thread");
    }

    pub async fn request_arbo(&self, to: Address) -> io::Result<bool> {
        let callback = self.callbacks.create(Callback::PullFs)?;

        self.to_network_message_tx
            .send(ToNetworkMessage::SpecificMessage(
                (MessageContent::RequestFs, None),
                vec![to],
            ))
            .expect("request_arbo: unable to update modification on the network thread");

        self.callbacks.async_wait_for(callback).await
    }

    pub fn edit_peer_ip(&self, actual: Address, new: Address) {
        log::info!("changing host {} to {}", actual, new);
        if let Some(mut peers) = self.peers.try_write_for(LOCK_TIMEOUT) {
            for peer in peers.iter_mut() {
                if peer.address == actual {
                    log::info!("done once");
                    peer.address = new.clone();
                }
            }
        }
    }

    pub fn send_arbo(&self, to: Address, global_config_bytes: Vec<u8>) -> io::Result<()> {
        let arbo = Arbo::read_lock(&self.arbo, "send_arbo")?;
        let mut entries = arbo.get_raw_entries();

        //Remove ignored entries
        entries.remove(&3u64);
        entries.entry(1u64).and_modify(|inode| {
            if let FsEntry::Directory(childrens) = &mut inode.entry {
                childrens.retain(|x| *x != 3u64);
            }
        });

        if let Some(peers) = self.peers.try_read_for(LOCK_TIMEOUT) {
            let peers_address_list = peers
                .iter()
                .filter_map(|peer| {
                    if peer.address != to {
                        Some(peer.address.clone())
                    } else {
                        None
                    }
                })
                .collect();

            self.to_network_message_tx
                .send(ToNetworkMessage::SpecificMessage(
                    (
                        MessageContent::FsAnswer(
                            FileSystemSerialized {
                                fs_index: entries,
                                next_inode: self.get_next_inode()?,
                            },
                            peers_address_list,
                            global_config_bytes,
                        ),
                        None,
                    ),
                    vec![to],
                ))
                .expect("send_arbo: unable to update modification on the network thread");
            Ok(())
        } else {
            Err(std::io::Error::new(
                io::ErrorKind::WouldBlock,
                "Deadlock while trying to read peers",
            ))
        }
    }

    pub fn register_new_node(&self, socket: Address, addr: Address) {
        self.edit_peer_ip(socket, addr);
    }

    pub fn disconnect_peer(&self, addr: Address) -> io::Result<()> {
        self.peers
            .try_write_for(LOCK_TIMEOUT)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::WouldBlock,
                format!("disconnect_peer: can't write lock peers"),
            ))?
            .retain(|p| p.address != addr);
        Ok(())
    }

    pub async fn network_airport(
        mut network_reception: UnboundedReceiver<FromNetworkMessage>,
        fs_interface: Arc<FsInterface>,
    ) {
        loop {
            let FromNetworkMessage { origin, content } = match network_reception.recv().await {
                Some(message) => message,
                None => continue,
            };
            log::debug!("message from {} : {:?}", origin, content);

            let action_result = match content.clone() { // remove scary clone
                MessageContent::PullAnswer(id, binary) => fs_interface.recept_binary(id, binary),
                MessageContent::RedundancyFile(id, binary) => fs_interface.recept_binary(id, binary),
                MessageContent::Inode(inode) => fs_interface.recept_inode(inode).or_else(|err| {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("WhError: {err}"),
                        ))
                    }),
                MessageContent::EditHosts(id, hosts) => fs_interface.recept_edit_hosts(id, hosts),
                MessageContent::AddHosts(id, hosts) => fs_interface.recept_add_hosts(id, hosts),
                MessageContent::RemoveHosts(id, hosts) => {
                    fs_interface.recept_remove_hosts(id, hosts)
                }
                MessageContent::EditMetadata(id, meta, host) => {
                    fs_interface.recept_edit_metadata(id, meta, host)
                }
                MessageContent::Remove(id) => fs_interface.recept_remove_inode(id).or_else(|err| {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("WhError: {err}"),
                        ))
                    }),
                MessageContent::RequestFile(inode, peer) => fs_interface.send_file(inode, peer),
                MessageContent::RequestFs => fs_interface.send_filesystem(origin),
                MessageContent::Register(addr) => Ok(fs_interface.register_new_node(origin, addr)),
                MessageContent::Rename(parent, new_parent, name, new_name) => {
                    fs_interface.accept_rename(parent, new_parent, &name, &new_name)
                }
                MessageContent::RequestPull(id) => fs_interface.pull_file_async(id).await,
                MessageContent::SetXAttr(ino, key, data) => fs_interface
                    .network_interface
                    .recept_inode_xattr(ino, key, data)
                    .or_else(|err| {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("WhError: {err}"),
                        ))
                    }),
                MessageContent::RemoveXAttr(ino, key) => fs_interface
                    .network_interface
                    .recept_remove_inode_xattr(ino, key)
                    .or_else(|err| {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("WhError: {err}"),
                        ))
                    }),
                MessageContent::FsAnswer(_, _, _) => {
                    Err(io::Error::new(ErrorKind::InvalidInput,
                        "Late answer from first connection, loaded network interface shouldn't recieve FsAnswer"))
                },
                MessageContent::Disconnect(addr) => fs_interface.network_interface.disconnect_peer(addr)
            };
            if let Err(error) = action_result {
                log::error!(
                    "Network airport couldn't operate operation {content:?}, error found: {error}"
                );
            }
        }
    }

    pub async fn contact_peers(
        peers_list: Arc<RwLock<Vec<PeerIPC>>>,
        mut rx: UnboundedReceiver<ToNetworkMessage>,
    ) {
        log::info!("contact peers");
        while let Some(message) = rx.recv().await {
            // geeting all peers network senders
            let peers_tx: Vec<(UnboundedSender<MessageAndStatus>, String)> = peers_list
                .try_read_for(LOCK_TIMEOUT)
                .expect("mutext error on contact_peers") // TODO - handle timeout
                .iter()
                .map(|peer| (peer.sender.clone(), peer.address.clone()))
                .collect();

            match message {
                ToNetworkMessage::BroadcastMessage(message_content) => {
                    peers_tx.iter().for_each(|(channel, address)| {
                        println!("peer: {}", address);
                        channel
                            .send((message_content.clone(), None))
                            .expect(&format!("failed to send message to peer {}", address))
                    });
                }
                ToNetworkMessage::SpecificMessage((message_content, status_tx), origins) => {
                    peers_tx
                        .iter()
                        .filter(|&(_, address)| origins.contains(address))
                        .for_each(|(channel, address)| {
                            channel
                                .send((message_content.clone(), status_tx.clone()))
                                .expect(&format!("failed to send message to peer {}", address))
                        });
                }
            };
        }
    }

    pub async fn incoming_connections_watchdog(
        server: Arc<Server>,
        nfa_tx: UnboundedSender<FromNetworkMessage>,
        existing_peers: Arc<RwLock<Vec<PeerIPC>>>,
    ) {
        while let Ok((stream, addr)) = server.listener.accept().await {
            log::debug!("GOT ADDRESS {addr}");
            let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("Error during the websocket handshake occurred");

            let (write, read) = futures_util::StreamExt::split(ws_stream);
            let new_peer =
                PeerIPC::connect_from_incomming(addr.to_string(), nfa_tx.clone(), write, read);
            {
                existing_peers
                    .try_write_for(LOCK_TIMEOUT)
                    .expect("incoming_connections_watchdog: can't lock existing peers")
                    .push(new_peer);
            }
        }
    }

    // !SECTION ^ Node related
}
