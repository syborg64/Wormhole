use std::{path::PathBuf, sync::Arc};

use openat::Dir;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{network::{message::NetworkMessage, peer_ipc::PeerIPC}, providers::FsIndex};

use super::network_interface::NetworkInterface;


// TODO
pub type PodConfig = u64;

pub struct Pod {
    network_interface: NetworkInterface,
    mount_point: PathBuf, // TODO - replace by Ludo's unipath
    peers: Vec<PeerIPC>,
    pod_conf: PodConfig,
}
