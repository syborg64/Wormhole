use std::{io, sync::Arc};

use crate::error::WhError;
#[cfg(target_os = "linux")]
use crate::fuse::fuse_impl::mount_fuse;
use crate::network::message::{
    Feedback, FileSystemSerialized, FromNetworkMessage, MessageContent, ToNetworkMessage,
};
#[cfg(target_os = "windows")]
use crate::winfsp::winfsp_impl::mount_fsp;
use custom_error::custom_error;
#[cfg(target_os = "linux")]
use fuser;
use log::info;
use parking_lot::RwLock;
use serde::Serialize;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
#[cfg(target_os = "windows")]
use winfsp::host::FileSystemHost;

use crate::network::{message::Address, peer_ipc::PeerIPC, server::Server};

use crate::pods::{
    arbo::{index_folder, Arbo},
    disk_manager::DiskManager,
    filesystem::fs_interface::FsInterface,
    network::network_interface::NetworkInterface,
    whpath::WhPath,
};

use super::arbo::{InodeId, ARBO_FILE_FNAME};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Pod {
    name: String,
    network_interface: Arc<NetworkInterface>,
    fs_interface: Arc<FsInterface>,
    mount_point: WhPath,
    peers: Arc<RwLock<Vec<PeerIPC>>>,
    #[cfg(target_os = "linux")]
    fuse_handle: fuser::BackgroundSession,
    #[cfg(target_os = "windows")]
    fsp_host: FileSystemHost<'static>,
    network_airport_handle: Option<JoinHandle<()>>,
    peer_broadcast_handle: Option<JoinHandle<()>>,
    new_peer_handle: Option<JoinHandle<()>>,
}

custom_error! {pub PodStopError
    WhError{source: WhError} = "{source}",
    KeyNotFound = "Key not found"
}

pub async fn initiate_connection(
    peers_addrs: Vec<Address>,
    server_address: Address,
    tx: &UnboundedSender<FromNetworkMessage>,
    rx: &mut UnboundedReceiver<FromNetworkMessage>,
) -> Option<(FileSystemSerialized, Vec<Address>, PeerIPC)> {
    if peers_addrs.len() >= 1 {
        for first_contact in peers_addrs {
            let first_ipc = PeerIPC::connect(first_contact.to_owned(), tx.clone()).await;

            if let Some(ipc) = first_ipc {
                if let Err(err) = ipc.sender.send((MessageContent::RequestFs, None)) {
                    info!(
                        "Connection with {first_contact} failed: {err}.\n
                        Trying with next know address"
                    );
                    continue;
                }

                loop {
                    match rx.recv().await {
                        Some(FromNetworkMessage {
                            origin: _,
                            content: MessageContent::FsAnswer(fs, mut peers_address),
                        }) => {
                            // remove itself from peers and first_connect because the connection is already existing
                            peers_address.retain(|address| {
                                *address != server_address && *address != first_contact
                            });
                            return Some((fs, peers_address, ipc));
                        }
                        Some(_) => {
                            info!(
                                "First message with {first_contact} failed: His answer is not the FileSystem, corrupted client.\n
                                Trying with next know address"
                            );
                            break;
                        }
                        None => continue,
                    };
                }
            }
        }
        info!("None of the known address answered correctly, starting a FS.")
    }
    None
}

fn register_to_others(peers: &Vec<PeerIPC>, self_address: &Address) -> std::io::Result<()> {
    for peer in peers {
        peer.sender
            .send((MessageContent::Register(self_address.clone()), None))
            .map_err(|err| std::io::Error::new(io::ErrorKind::NotConnected, err))?;
    }
    Ok(())
}

impl Pod {
    pub async fn new(
        name: String,
        mount_point: WhPath,
        mut know_peers: Vec<Address>,
        server: Arc<Server>,
        server_address: Address,
    ) -> io::Result<Self> {
        log::info!("mount point {}", mount_point);
        let (mut arbo, next_inode) =
            index_folder(&mount_point, &server_address).expect("unable to index folder");
        let (to_network_message_tx, to_network_message_rx) = mpsc::unbounded_channel();
        let (from_network_message_tx, mut from_network_message_rx) = mpsc::unbounded_channel();

        know_peers.retain(|x| *x != server_address);

        let mut peers = vec![];

        if let Some((fs_serialized, peers_addrs, ipc)) = initiate_connection(
            know_peers,
            server_address.clone(),
            &from_network_message_tx,
            &mut from_network_message_rx,
        )
        .await
        {
            peers = PeerIPC::peer_startup(peers_addrs, from_network_message_tx.clone()).await;
            peers.push(ipc);
            register_to_others(&peers, &server_address)?;
            arbo.overwrite_self(fs_serialized.fs_index);
        }

        let arbo: Arc<RwLock<Arbo>> = Arc::new(RwLock::new(arbo));

        let network_interface = Arc::new(NetworkInterface::new(
            arbo.clone(),
            mount_point.clone(),
            to_network_message_tx,
            next_inode,
            Arc::new(RwLock::new(peers)),
            server_address,
        ));

        let disk_manager = DiskManager::new(mount_point.clone())?;
        let fs_interface = Arc::new(FsInterface::new(
            network_interface.clone(),
            disk_manager,
            arbo.clone(),
        ));

        // Start ability to recieve messages
        let network_airport_handle = Some(tokio::spawn(NetworkInterface::network_airport(
            from_network_message_rx,
            fs_interface.clone(),
        )));

        // Start ability to send messages
        let peer_broadcast_handle = Some(tokio::spawn(NetworkInterface::contact_peers(
            network_interface.peers.clone(),
            to_network_message_rx,
        )));

        let new_peer_handle = Some(tokio::spawn(
            NetworkInterface::incoming_connections_watchdog(
                server,
                from_network_message_tx,
                network_interface.peers.clone(),
            ),
        ));

        let peers = network_interface.peers.clone();

        Ok(Self {
            name: name.clone(),
            network_interface,
            fs_interface: fs_interface.clone(),
            mount_point: mount_point.clone(),
            peers,
            #[cfg(target_os = "linux")]
            fuse_handle: mount_fuse(&mount_point, fs_interface.clone())?,
            #[cfg(target_os = "windows")]
            fsp_host: mount_fsp(&mount_point, fs_interface.clone())?,
            network_airport_handle,
            peer_broadcast_handle,
            new_peer_handle,
        })
    }

    /// for a given file, will try to send it to one host,
    /// trying each until succes
    /// log a warn on failure
    fn send_file_to_possible_hosts(
        &self,
        possible_hosts: &Vec<Address>,
        ino: InodeId,
        path: WhPath,
    ) {
        let mut host_nb = 0;

        loop {
            if possible_hosts.len() <= host_nb {
                log::warn!("Pod::stop no hosts can receive this file: {path}");
                // TODO - while merge between pods is not implemented, the file is untracked
                // (present on disk, but not tracked by wormhole, and not deleted either)
            }
            let file_content = self
                .fs_interface
                .disk
                .read_file_to_end(path.clone())
                .expect("Pod::stop: unable to read file from disk");

            let (feedback_tx, mut feedback_rx) = tokio::sync::mpsc::unbounded_channel::<Feedback>();

            self.network_interface
                .to_network_message_tx
                .send(ToNetworkMessage::SpecificMessage(
                    (
                        MessageContent::PullAnswer(ino, file_content),
                        Some(feedback_tx.clone()),
                    ),
                    vec![possible_hosts[0].clone()],
                ))
                .unwrap();

            match feedback_rx.blocking_recv().unwrap() {
                Feedback::Sent => return,
                Feedback::Error => host_nb += 1,
            };
        }
    }

    fn files_to_send_when_stopping(&self) -> Result<Vec<(InodeId, WhPath)>, PodStopError> {
        let hosted_only_by_me = self
            .network_interface
            .files_hosted_only_by(&self.network_interface.self_addr)?;

        let arbo = Arbo::n_read_lock(
            &self.network_interface.arbo,
            "Pod::files_to_send_when_stopping",
        )?;

        Ok(hosted_only_by_me
            .into_iter()
            .filter_map(|inode| {
                Some((
                    inode.id,
                    arbo.n_get_path_from_inode_id(inode.id)
                        .map_err(|e| log::error!("Pod::stop(2): {e}"))
                        .ok()?,
                ))
            })
            .collect())
    }

    pub fn stop(&self) -> Result<(), PodStopError> {
        // NOTE
        // in actual state, all operations (request from network other than just pulling the asked files)
        // made after calling this function but before dropping the pod are undefined behavior.

        // drop(self.fuse_handle); // FIXME - do something like block the filesystem

        let peers: Vec<Address> = self
            .peers
            .read()
            .iter()
            .map(|peer| peer.address.clone())
            .collect();

        let files_to_send = self.files_to_send_when_stopping()?;

        files_to_send.into_iter().for_each(|(id, path)| {
            self.send_file_to_possible_hosts(&peers, id, path.clone());
        });

        let arbo = Arbo::read_lock(&self.network_interface.arbo, "Pod::stop(3)")
            .expect("Pod::stop arbo read lock");
        let _ = self.fs_interface.disk.remove_file(ARBO_FILE_FNAME.into());
        self.fs_interface
            .disk
            .write_file(
                ARBO_FILE_FNAME.into(),
                &bincode::serialize(&*arbo).expect("can't serialize arbo to bincode"),
                0,
            )
            .expect("can't write arbo to disk");
        Ok(())
    }
}
