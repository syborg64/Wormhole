use std::{io, sync::Arc};

use parking_lot::RwLock;
use tokio::sync::mpsc;

use crate::{
    fuse::fuse_impl::mount_fuse,
    network::{message::Address, peer_ipc::PeerIPC, server::Server},
    providers::whpath::WhPath,
};

use super::{
    arbo::Arbo,
    disk_manager::{self, DiskManager},
    fs_interface::{self, FsInterface},
    network_interface::NetworkInterface,
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
    ) -> io::Result<Self> {
        let arbo: Arc<RwLock<Arbo>> = Arc::new(RwLock::new(Arbo::new()));
        let (to_network_message_tx, to_network_message_rx) = mpsc::unbounded_channel();
        let (from_network_message_tx, from_network_message_rx) = mpsc::unbounded_channel();

        let network_interface = Arc::new(NetworkInterface::new(
            arbo.clone(),
            mount_point.clone(),
            to_network_message_tx,
            0,
        ));

        let disk_manager = DiskManager::new(mount_point.clone())?;

        let fs_interface = Arc::new(FsInterface::new(
            network_interface.clone(),
            disk_manager,
            arbo.clone(),
        ));

        network_interface.start_network_airport(
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
