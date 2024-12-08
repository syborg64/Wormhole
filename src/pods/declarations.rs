use std::{path::PathBuf, sync::Arc};

use openat::Dir;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{network::{message::NetworkMessage, peer_ipc::PeerIPC}, providers::FsIndex};


// TODO
type PodConfig = u64;

struct Pod {
    logical_manager: LogicalManager,
    mount_point: PathBuf, // TODO - replace by Ludo's unipath
    peers: Vec<PeerIPC>,
    pod_conf: PodConfig,
}

struct LogicalManager {
    arbo: FsIndex,
    mount_point: PathBuf, // TODO - replace by Ludo's unipath
    disk: Arc<DiskManager>,
    network_sender: UnboundedSender<NetworkMessage>,
    next_inode: u64, // TODO - replace with Ino type
    network_airport_handle: JoinHandle<()>,
}


struct DiskManager {
    handle: Dir,
}