use std::{collections::HashMap, io, sync::Arc};

use parking_lot::{Mutex, RwLock};
use tokio::sync::{
    broadcast,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

use super::{
    arbo::{FsEntry, Metadata},
    whpath::WhPath,
};
use crate::network::{
    message::{
        self, Address, FileSystemSerialized, FromNetworkMessage, MessageContent, ToNetworkMessage,
    },
    peer_ipc::PeerIPC,
    server::Server,
};

use super::{
    arbo::{Arbo, Inode, InodeId, LOCK_TIMEOUT},
    fs_interface::FsInterface,
};

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub enum Callback {
    Pull(InodeId),
    PullFs,
}

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

pub struct NetworkInterface {
    pub arbo: Arc<RwLock<Arbo>>,
    pub mount_point: WhPath,
    pub to_network_message_tx: UnboundedSender<ToNetworkMessage>,
    pub next_inode: Mutex<InodeId>, // TODO - replace with InodeIndex type
    pub callbacks: Callbacks,
    pub peers: Arc<RwLock<Vec<PeerIPC>>>,
    pub self_addr: Address,
}

impl NetworkInterface {
    pub fn new(
        arbo: Arc<RwLock<Arbo>>,
        mount_point: WhPath,
        to_network_message_tx: UnboundedSender<ToNetworkMessage>,
        next_inode: InodeId,
        peers: Arc<RwLock<Vec<PeerIPC>>>,
        self_addr: Address,
    ) -> Self {
        let next_inode = Mutex::new(next_inode);

        Self {
            arbo,
            mount_point,
            to_network_message_tx,
            next_inode,
            callbacks: Callbacks {
                callbacks: HashMap::new().into(),
            },
            peers,
            self_addr,
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

    #[must_use]
    /// add the requested entry to the arbo and inform the network
    pub fn register_new_file(&self, inode: Inode) -> io::Result<u64> {
        let new_inode_id = inode.id;

        if let Some(mut arbo) = self.arbo.try_write_for(LOCK_TIMEOUT) {
            arbo.add_inode(inode.clone())?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "mkfile: can't write-lock arbo's RwLock",
            ));
        };

        // TODO - add myself to hosts

        if new_inode_id != 3u64 {
            self.to_network_message_tx
                .send(ToNetworkMessage::BroadcastMessage(
                    message::MessageContent::Inode(inode, new_inode_id),
                ))
                .expect("mkfile: unable to update modification on the network thread");
        }
        // TODO - if unable to update for some reason, should be passed to the background worker

        Ok(new_inode_id)
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
    pub fn acknowledge_new_file(&self, inode: Inode, _id: InodeId) -> io::Result<()> {
        if let Some(mut arbo) = self.arbo.try_write_for(LOCK_TIMEOUT) {
            match arbo.add_inode(inode) {
                Ok(()) => (),
                Err(_) => todo!("acknowledge_new_file: file already existing: conflict"), // TODO
            };
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "mkfile: can't write-lock arbo's RwLock",
            ));
        };

        Ok(())
    }

    /// remove the requested entry to the arbo and inform the network
    pub fn unregister_file(&self, id: InodeId) -> io::Result<Inode> {
        let removed_inode: Inode;

        if let Some(mut arbo) = self.arbo.try_write_for(LOCK_TIMEOUT) {
            removed_inode = arbo.remove_inode(id)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "mkfile: can't write-lock arbo's RwLock",
            ));
        };

        if id != 3u64 {
            self.to_network_message_tx
                .send(ToNetworkMessage::BroadcastMessage(
                    message::MessageContent::Remove(id),
                ))
                .expect("mkfile: unable to update modification on the network thread");
        }

        // TODO - if unable to update for some reason, should be passed to the background worker

        Ok(removed_inode)
    }

    pub fn acknowledge_unregister_file(&self, id: InodeId) -> io::Result<Inode> {
        let removed_inode: Inode;

        if let Some(mut arbo) = self.arbo.try_write_for(LOCK_TIMEOUT) {
            removed_inode = arbo.remove_inode(id)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "mkfile: can't write-lock arbo's RwLock",
            ));
        };

        Ok(removed_inode)
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
    pub fn pull_file(&self, file: InodeId) -> io::Result<Option<Callback>> {
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

            self.to_network_message_tx
                .send(ToNetworkMessage::SpecificMessage(
                    message::MessageContent::RequestFile(file, self.self_addr.clone()),
                    vec![hosts[0].clone()], // NOTE - naive choice for now
                ))
                .expect("pull_file: unable to request on the network thread");

            Ok(Some(callback))
        }
    }

    pub fn send_file(&self, inode: InodeId, data: Vec<u8>, to: Address) -> io::Result<()> {
        self.to_network_message_tx
            .send(ToNetworkMessage::SpecificMessage(
                MessageContent::PullAnswer(inode, data),
                vec![to],
            ))
            .expect("send_file: unable to update modification on the network thread");
        Ok(())
    }

    pub fn revoke_remote_hosts(&self, id: InodeId) -> io::Result<()> {
        self.to_network_message_tx
            .send(ToNetworkMessage::BroadcastMessage(
                MessageContent::EditHosts(id, vec![self.self_addr.clone()]),
            ))
            .expect("revoke_remote_hosts: unable to update modification on the network thread");
        Ok(())
        /* REVIEW
         * This system (and others broadcasts systems) should be reviewed as they don't check success.
         * In this case, if another host misses this order, it will not update it's file.
         * We could create a "broadcast" callback with the number of awaited confirmations and a timeout
         * before resend or fail declaration.
         * Or send a bunch of Specific messages
         */
    }

    pub fn update_remote_hosts(&self, inode: &Inode) -> io::Result<()> {
        if let FsEntry::File(hosts) = &inode.entry {
            self.to_network_message_tx
                .send(ToNetworkMessage::BroadcastMessage(
                    MessageContent::EditHosts(inode.id, hosts.clone()),
                ))
                .expect("update_remote_hosts: unable to update modification on the network thread");
            Ok(())
        } else {
            Err(io::ErrorKind::InvalidInput.into())
        }
    }

    pub fn update_metadata(&self, id: InodeId, meta: Metadata) -> io::Result<()> {
        let mut arbo = Arbo::write_lock(&self.arbo, "fs_interface::get_inode_attributes")?;
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
                MessageContent::RequestFs,
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

    pub fn send_arbo(&self, to: Address) -> io::Result<()> {
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
            let peers_address_list = peers.iter().map(|peer| peer.address.clone()).collect();

            self.to_network_message_tx
                .send(ToNetworkMessage::SpecificMessage(
                    MessageContent::FsAnswer(
                        FileSystemSerialized {
                            fs_index: entries,
                            next_inode: self.get_next_inode()?,
                        },
                        peers_address_list,
                    ),
                    vec![to],
                ))
                .expect("send_arbo: unable to update modification on the network thread");
            Ok(())
        } else {
            Err(std::io::Error::new(
                io::ErrorKind::Deadlock,
                "Deadlock while trying to read peers",
            ))
        }
    }

    pub fn register_new_node(&self, socket: Address, addr: Address) {
        self.edit_peer_ip(socket, addr);
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
            // log::debug!("message from {} : {:?}", origin, content);

            let action_result = match content {
                MessageContent::PullAnswer(id, binary) => fs_interface.recept_binary(id, binary),
                MessageContent::Inode(inode, id) => fs_interface.recept_inode(inode, id),
                MessageContent::EditHosts(id, hosts) => fs_interface.recept_edit_hosts(id, hosts),
                MessageContent::EditMetadata(id, meta, host) => {
                    fs_interface.recept_edit_metadata(id, meta, host)
                }
                MessageContent::Remove(id) => fs_interface.recept_remove_inode(id),
                MessageContent::RequestFile(inode, peer) => fs_interface.send_file(inode, peer),
                MessageContent::RequestFs => fs_interface.send_filesystem(origin),
                MessageContent::Register(addr) => Ok(fs_interface.register_new_node(origin, addr)),
                MessageContent::Rename(parent, new_parent, name, new_name) => {
                    fs_interface.accept_rename(parent, new_parent, &name, &new_name)
                }
                MessageContent::FsAnswer(_, _) => {
                    todo!("Late answer from first connection");
                }
            };
            if let Err(error) = action_result {
                log::error!("Network airport couldn't operate this operation: {error}");
            }
        }
    }

    pub async fn contact_peers(
        peers_list: Arc<RwLock<Vec<PeerIPC>>>,
        mut rx: UnboundedReceiver<ToNetworkMessage>,
    ) {
        // on message reception, broadcast it to all peers senders
        while let Some(message) = rx.recv().await {
            let peer_tx: Vec<(UnboundedSender<MessageContent>, String)> = peers_list
                .try_read_for(LOCK_TIMEOUT)
                .unwrap() // TODO - handle timeout
                .iter()
                .map(|peer| (peer.sender.clone(), peer.address.clone()))
                .collect();

            println!("broadcasting message to peers:\n{:?}", message);
            log::info!(
                "peers list {:#?}",
                peers_list
                    .read()
                    .iter()
                    .map(|peer| peer.address.clone())
                    .collect::<Vec<String>>()
            );
            match message {
                ToNetworkMessage::BroadcastMessage(message_content) => {
                    peer_tx.iter().for_each(|(channel, address)| {
                        println!("peer: {}", address);
                        channel
                            .send(message_content.clone())
                            .expect(&format!("failed to send message to peer {}", address))
                    });
                }
                ToNetworkMessage::SpecificMessage(message_content, origins) => {
                    peer_tx
                        .iter()
                        .filter(|&(_, address)| origins.contains(address))
                        .for_each(|(channel, address)| {
                            channel
                                .send(message_content.clone())
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
        while let Ok((stream, _)) = server.listener.accept().await {
            let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("Error during the websocket handshake occurred");
            let addr = ws_stream.get_ref().peer_addr().unwrap().to_string();

            let (write, read) = futures_util::StreamExt::split(ws_stream);
            let new_peer = PeerIPC::connect_from_incomming(addr, nfa_tx.clone(), write, read);
            {
                existing_peers
                    .try_write_for(LOCK_TIMEOUT)
                    .expect("incoming_connections_watchdog: can't lock existing peers")
                    .push(new_peer);
            }
        }
    }
}
