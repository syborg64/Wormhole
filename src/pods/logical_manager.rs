use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use fuser::FileType;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{network::message::{self, NetworkMessage}, providers::FsIndex};

use super::disk_manager::DiskManager;

pub struct LogicalManager {
    pub arbo: Arc<Mutex<FsIndex>>,
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

    /// Get a new inode, add the requested entry to the arbo and inform the network
    pub fn register_new_file(&self, ftype: FileType, path: PathBuf) -> u64 {
        let mut arbo = self.arbo.lock().expect("arbo lock error");
        let ino = self.get_next_inode();

        arbo.insert(ino, (ftype, path.clone()));
        self.network_sender
            .send(NetworkMessage::File(message::File {
                path,
                file: [].to_vec(), // REVIEW - why this field ? useful ?
                ino: ino,
            }))
            .expect("mkfile: unable to update modification on the network thread");

        ino
    }
}
