use std::{collections::HashMap, io, sync::Arc};

use parking_lot::RwLock;
use tokio::sync::mpsc;

use crate::{
    network::{message::Address, peer_ipc::PeerIPC},
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
    mount_point: WhPath, // TODO - replace by Ludo's unipath
    peers: Vec<PeerIPC>,
    pod_conf: PodConfig,
    fuse_handle: u64, //fuser::BackgroundSession,
}

impl Pod {
    pub fn new(mount_point: WhPath, config: PodConfig, peers: Vec<Address>) -> io::Result<Self> {
        let arbo: Arc<RwLock<Arbo>> = Arc::new(RwLock::new(Arbo::new()));
        let (to_external_tx, to_external_rx) = mpsc::unbounded_channel();
        let (from_external_tx, from_external_rx) = mpsc::unbounded_channel();

        let network_interface = Arc::new(NetworkInterface::new(
            arbo.clone(),
            mount_point.clone(),
            to_external_tx,
            0,
        ));

        let disk_manager = DiskManager::new(mount_point.clone())?;

        let fs_interface = Arc::new(FsInterface::new(
            network_interface.clone(),
            disk_manager,
            arbo.clone(),
        ));

        network_interface.start_network_airport(fs_interface.clone(), from_external_rx);

        Ok(Self {
            network_interface,
            fs_interface,
            mount_point,
            peers: vec![],
            pod_conf: 0,
            fuse_handle: 0,
        })
    }
}
