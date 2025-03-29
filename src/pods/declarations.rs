use std::{io, sync::Arc};

use parking_lot::RwLock;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

use crate::network::message::FileSystemSerialized;
use crate::{
    fuse::fuse_impl::mount_fuse,
    network::{
        message::{Address, FromNetworkMessage, MessageContent},
        peer_ipc::PeerIPC,
        server::Server,
    },
};

use super::{
    arbo::{index_folder, Arbo},
    disk_manager::DiskManager,
    fs_interface::FsInterface,
    network_interface::NetworkInterface,
    whpath::WhPath,
};

// TODO
pub type PodConfig = u64;

#[allow(dead_code)]
pub struct Pod {
    network_interface: Arc<NetworkInterface>,
    fs_interface: Arc<FsInterface>,
    mount_point: WhPath,
    peers: Arc<RwLock<Vec<PeerIPC>>>,
    pod_conf: PodConfig,
    fuse_handle: fuser::BackgroundSession,
    network_airport_handle: Option<JoinHandle<()>>,
    peer_broadcast_handle: Option<JoinHandle<()>>,
    new_peer_handle: Option<JoinHandle<()>>,
}

pub async fn initiate(
    peers_addrs: Vec<Address>,
    server_address: Address,
    tx: &UnboundedSender<FromNetworkMessage>,
    rx: &mut UnboundedReceiver<FromNetworkMessage>,
) -> io::Result<Option<(FileSystemSerialized, Vec<Address>, PeerIPC)>> {
    if peers_addrs.len() >= 1 {
        for first_contact in peers_addrs {
            let first_ipc = PeerIPC::connect(first_contact.to_owned(), tx.clone()).await;

            if let Some(ipc) = first_ipc {
                let _ = ipc
                    .sender
                    .send(MessageContent::RequestFs(server_address.clone()));

                loop {
                    let FromNetworkMessage { origin: _, content } = match rx.recv().await {
                        Some(message) => message,
                        None => continue,
                    };

                    if let MessageContent::FsAnswer(fs_serialized, mut network_address) = content {
                        network_address.retain(|address| {
                            *address != server_address && *address != first_contact
                        });
                        return Ok(Some((fs_serialized, network_address, ipc)));
                    } else {
                        return Err(std::io::Error::new(
                            io::ErrorKind::Unsupported,
                            "First message from other is not fs answer",
                        ));
                    }
                }
            }
        }
        Ok(None)
    } else {
        Ok(None)
    }
}

impl Pod {
    pub async fn new(
        mount_point: WhPath,
        config: PodConfig,
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

        if let Some((fs_serialized, peers_addrs, ipc)) = initiate(
            know_peers,
            server_address.clone(),
            &from_network_message_tx,
            &mut from_network_message_rx,
        )
        .await?
        {
            // remove itself from peers and first_connect because the connection is already existing
            peers = PeerIPC::peer_startup(peers_addrs, from_network_message_tx.clone()).await;
            peers.push(ipc);
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
            network_interface,
            fs_interface: fs_interface.clone(),
            mount_point: mount_point.clone(),
            peers,
            pod_conf: config,
            fuse_handle: mount_fuse(&mount_point, fs_interface.clone())?,
            network_airport_handle,
            peer_broadcast_handle,
            new_peer_handle,
        })
    }
}
