use std::{path::PathBuf, sync::Arc};

use openat::Dir;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{network::{message::NetworkMessage, peer_ipc::PeerIPC}, providers::FsIndex};


// TODO
pub type PodConfig = u64;

pub struct Pod {
    logical_manager: LogicalManager,
    mount_point: PathBuf, // TODO - replace by Ludo's unipath
    peers: Vec<PeerIPC>,
    pod_conf: PodConfig,
}
