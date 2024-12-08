use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{network::message::NetworkMessage, providers::FsIndex};

use super::disk_manager::DiskManager;

pub struct LogicalManager {
    pub arbo: Arc<FsIndex>,
    pub mount_point: PathBuf, // TODO - replace by Ludo's unipath
    pub disk: Arc<DiskManager>,
    pub network_sender: UnboundedSender<NetworkMessage>,
    pub next_inode: Arc<Mutex<u64>>, // TODO - replace with Ino type
    pub network_airport_handle: JoinHandle<()>,
}

impl LogicalManager {
    pub fn get_next_inode(&self) -> u64 {
        let mut inode = self.next_inode.lock().expect("unable to lock inode mutex");
        let available_inode = *inode;
        *inode += 1;
        available_inode
    }
}
