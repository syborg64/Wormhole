use tokio::sync::mpsc;

use crate::{network::{message::Address, peer_ipc::PeerIPC}, providers::whpath::WhPath};

use super::{arbo::Arbo, network_interface::NetworkInterface};

// TODO
pub type PodConfig = u64;

pub struct Pod {
    network_interface: NetworkInterface,
    mount_point: WhPath, // TODO - replace by Ludo's unipath
    peers: Vec<PeerIPC>,
    pod_conf: PodConfig,
    fuse_handle: fuser::BackgroundSession,
}

impl Pod {
    pub fn new(mount_point: WhPath, config: PodConfig, peers: Vec<Address>) -> Self {
        let arbo: Arbo;
        let (network_sender, network_reception) = mpsc::unbounded_channel();

        let network_interface = NetworkInterface::new(
            arbo,
            mount_point.clone(),
            network_sender,
            network_reception,
            0,
            fs_interface)
    }
}