use std::{io, sync::Arc};

#[cfg(target_os = "linux")]
use fuser;
use log::info;
use parking_lot::RwLock;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
#[cfg(target_os = "windows")]
use winfsp::host::FileSystemHost;

use crate::config::GlobalConfig;
#[cfg(target_os = "linux")]
use crate::fuse::fuse_impl::mount_fuse;
use crate::network::message::{FileSystemSerialized, FromNetworkMessage, MessageContent};
#[cfg(target_os = "linux")]
use crate::pods::disk_manager::unix_disk_manager::UnixDiskManager;
// #[cfg(target_os = "windows")]
// use crate::pods::disk_manager::windows_disk_manager::WindowsDiskManager;
use crate::pods::disk_manager::dummy_disk_manager::DummyDiskManager;
#[cfg(target_os = "windows")]
use crate::winfsp::winfsp_impl::mount_fsp;

use crate::network::{message::Address, peer_ipc::PeerIPC, server::Server};

use crate::pods::{
    arbo::{index_folder, Arbo},
    filesystem::fs_interface::FsInterface,
    network::network_interface::NetworkInterface,
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
    #[cfg(target_os = "linux")]
    fuse_handle: fuser::BackgroundSession,
    #[cfg(target_os = "windows")]
    fsp_host: FileSystemHost<'static>,
    network_airport_handle: Option<JoinHandle<()>>,
    peer_broadcast_handle: Option<JoinHandle<()>>,
    new_peer_handle: Option<JoinHandle<()>>,
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
        global_config: GlobalConfig,
        mount_point: WhPath,
        config: PodConfig,
        server: Arc<Server>,
        server_address: Address,
    ) -> io::Result<Self> {
        let mut global_config = global_config;

        log::info!("mount point {}", mount_point);
        let (mut arbo, mut next_inode) =
            index_folder(&mount_point, &server_address).expect("unable to index folder");
        let (to_network_message_tx, to_network_message_rx) = mpsc::unbounded_channel();
        let (from_network_message_tx, mut from_network_message_rx) = mpsc::unbounded_channel();

        global_config.general.peers.retain(|x| *x != server_address);

        let mut peers = vec![];

        if let Some((fs_serialized, peers_addrs, ipc)) = initiate_connection(
            global_config.general.peers,
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
            for index in arbo.get_raw_entries().keys() {
                // TEMP FIX FOR MERGE
                if *index > next_inode {
                    next_inode = *index;
                }
            }
            next_inode += 1;
        }

        let arbo: Arc<RwLock<Arbo>> = Arc::new(RwLock::new(arbo));

        let network_interface = Arc::new(NetworkInterface::new(
            arbo.clone(),
            mount_point.clone(),
            to_network_message_tx,
            next_inode,
            Arc::new(RwLock::new(peers)),
            server_address,
            global_config.redundancy.number,
        ));
        #[cfg(target_os = "linux")]
        let disk_manager = Box::new(UnixDiskManager::new(&mount_point)?);
        #[cfg(target_os = "windows")]
        let disk_manager = Box::new(DummyDiskManager::new(&mount_point)?);
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
            #[cfg(target_os = "linux")]
            fuse_handle: mount_fuse(&mount_point, fs_interface.clone())?,
            #[cfg(target_os = "windows")]
            fsp_host: mount_fsp(&mount_point, fs_interface.clone())?,
            network_airport_handle,
            peer_broadcast_handle,
            new_peer_handle,
        })
    }
}
