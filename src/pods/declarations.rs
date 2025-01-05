use std::{io, sync::Arc};

use parking_lot::RwLock;
use tokio::sync::mpsc;

use crate::{
    fuse::fuse_impl::mount_fuse,
    network::{message::Address, peer_ipc::PeerIPC, server::Server},
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

pub struct Pod {
    network_interface: Arc<NetworkInterface>,
    fs_interface: Arc<FsInterface>,
    mount_point: WhPath,
    peers: Vec<PeerIPC>,
    pod_conf: PodConfig,
    fuse_handle: fuser::BackgroundSession,
}

impl Pod {
    pub async fn new(
        mount_point: WhPath,
        config: PodConfig,
        peers: Vec<Address>,
        server: Arc<Server>,
        server_address: Address,
    ) -> io::Result<Self> {
        let (arbo, next_inode) = index_folder(&mount_point)?;
        let arbo: Arc<RwLock<Arbo>> = Arc::new(RwLock::new(arbo));
        let (to_network_message_tx, to_network_message_rx) = mpsc::unbounded_channel();
        let (from_network_message_tx, from_network_message_rx) = mpsc::unbounded_channel();

        let mut network_interface = Arc::new(NetworkInterface::new(
            arbo.clone(),
            mount_point.clone(),
            to_network_message_tx,
            next_inode,
            server_address
        ));

        let disk_manager = DiskManager::new(mount_point.clone())?;

        let fs_interface = Arc::new(FsInterface::new(
            network_interface.clone(),
            disk_manager,
            arbo.clone(),
        ));

        let mut_nwi =
            Arc::get_mut(&mut network_interface).expect("error in declarations.rs (arc)");
        /* NOTE
            If the Arc::get_mut does not work, the best options is probably to use Arc::get_mut_unchecked
            in an unsafe block.
            If not unsafe block are allowed, then we must use a mutex for the whole lifetime of network_interface
            even though this will never be mutated again
        */

        mut_nwi.start_network_airport(
            fs_interface.clone(),
            from_network_message_rx,
            from_network_message_tx.clone(),
            to_network_message_rx,
            server,
        );

        Ok(Self {
            network_interface,
            fs_interface: fs_interface.clone(),
            mount_point: mount_point.clone(),
            peers: PeerIPC::peer_startup(peers, from_network_message_tx).await,
            pod_conf: config,
            fuse_handle: mount_fuse(&mount_point, fs_interface.clone())?,
        })
    }
}
