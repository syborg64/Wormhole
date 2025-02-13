use std::{collections::HashMap, io, sync::Arc};

use log::{error, info};
use parking_lot::{Mutex, RwLock};
use tokio::sync::{
    broadcast,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

use super::{arbo::FsEntry, whpath::WhPath};
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

#[derive(Eq, Hash, PartialEq, Clone, Copy)]
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
        log::error!("RESOLVING CALLBACK");
        if let Some(mut callbacks) = self.callbacks.try_write_for(LOCK_TIMEOUT) {
            if let Some(cb) = callbacks.remove(&call) {
                cb.send(status).map_err(|error| {
                    std::io::Error::new(io::ErrorKind::AddrNotAvailable, error.to_string())
                })?;
                Ok(())
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
    self_addr: Address,
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

        self.to_network_message_tx
            .send(ToNetworkMessage::BroadcastMessage(
                message::MessageContent::Inode(inode, new_inode_id),
            ))
            .expect("mkfile: unable to update modification on the network thread");
        // TODO - if unable to update for some reason, should be passed to the background worker

        Ok(new_inode_id)
    }

    #[must_use]
    /// Get a new inode, add the requested entry to the arbo and inform the network
    pub fn acknowledge_new_file(&self, inode: Inode, id: InodeId) -> io::Result<()> {
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

        self.to_network_message_tx
            .send(ToNetworkMessage::BroadcastMessage(
                message::MessageContent::Remove(id),
            ))
            .expect("mkfile: unable to update modification on the network thread");

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

        if hosts.contains(&self.self_addr) {
            // if the asked file is already on disk
            Ok(None)
        } else {
            let callback = self.callbacks.create(Callback::Pull(file))?;

            self.to_network_message_tx
                .send(ToNetworkMessage::SpecificMessage(
                    message::MessageContent::RequestFile(file),
                    vec![hosts[0].clone()], // NOTE - naive choice for now
                ))
                .expect("pull_file: unable to request on the network thread");

            Ok(Some(callback))
        }
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

    pub fn send_arbo(&self, to: Address) -> io::Result<()> {
        let arbo = Arbo::read_lock(&self.arbo, "send_arbo")?;
        self.to_network_message_tx
            .send(ToNetworkMessage::SpecificMessage(
                MessageContent::FsAnswer(FileSystemSerialized {
                    fs_index: arbo.get_raw_entries(),
                    next_inode: self.get_next_inode()?,
                }),
                vec![to],
            ))
            .expect("send_arbo: unable to update modification on the network thread");
        Ok(())
    }

    // NOTE - meant only for pulling the arbo at startup !
    // Does not care for currently ongoing business when called
    pub fn replace_arbo(&self, new: FileSystemSerialized) -> io::Result<()> {
        log::error!("REPLACE ARBO");
        let mut arbo = Arbo::write_lock(&self.arbo, "replace_arbo")?;
        arbo.overwrite_self(new.fs_index);

        let mut next_inode = self
            .next_inode
            .try_lock_for(LOCK_TIMEOUT)
            .expect("couldn't lock next_inode");
        *next_inode = new.next_inode;

        // resolve callback :
        log::error!("REPLACE ARBO2");
        self.callbacks.resolve(Callback::PullFs, true);
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
            log::error!("airport {:#?}", content);

            match content {
                MessageContent::PullAnswer(id, binary) => {
                    fs_interface.recept_binary(id, binary);
                }
                MessageContent::Binary(bin) => {
                    println!("peer: {:?}", String::from_utf8(bin).unwrap_or_default());
                }
                MessageContent::Inode(inode, id) => {
                    fs_interface.recept_inode(inode, id);
                }
                MessageContent::EditHosts(id, hosts) => {
                    fs_interface.recept_edit_hosts(id, hosts);
                }
                MessageContent::Remove(ino) => {
                    todo!();
                    //let mut provider = provider.lock().expect("failed to lock mutex");
                    //provider.recpt_remove(ino);
                }
                MessageContent::Write(ino, data) => {
                    todo!();
                    // deprecated ?
                    //let mut provider = provider.lock().expect("failed to lock mutex");
                    //provider.recpt_write(ino, data);
                }
                MessageContent::Meta(_) => {}
                MessageContent::RequestFile(_) => {}
                MessageContent::RequestFs => {
                    fs_interface.send_filesystem(origin);
                }
                MessageContent::FsAnswer(fs) => {
                    log::error!("MSGCONTENT FSANSWER");
                    fs_interface.replace_arbo(fs);
                }
            };
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
            info!(
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
                            error!("here to {:?}", address);
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
            log::error!("new connected peer addr is {}", addr);

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
