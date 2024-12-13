use crate::{network::peer_ipc::PeerIPC, providers::whpath::WhPath};

use super::network_interface::NetworkInterface;

// TODO
pub type PodConfig = u64;

pub struct Pod {
    network_interface: NetworkInterface,
    mount_point: WhPath, // TODO - replace by Ludo's unipath
    peers: Vec<PeerIPC>,
    pod_conf: PodConfig,
}
