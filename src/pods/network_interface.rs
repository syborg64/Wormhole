use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{
    network::message::{self, ToNetworkMessage},
    providers::{FsEntry, FsIndex},
};

use super::disk_manager::DiskManager;

pub struct NetworkInterface {
    pub arbo: Arc<Mutex<FsIndex>>,
    pub mount_point: PathBuf, // TODO - replace by Ludo's unipath
    pub disk: Arc<DiskManager>,
    pub network_sender: UnboundedSender<ToNetworkMessage>,
    pub next_inode: Arc<Mutex<u64>>, // TODO - replace with InodeIndex type
    pub network_airport_handle: JoinHandle<()>,
}

impl NetworkInterface {
    pub fn get_next_inode(&self) -> u64 {
        let mut inode = self.next_inode.lock().expect("unable to lock inode mutex");
        let available_inode = *inode;
        *inode += 1;
        available_inode
    }

    /// Get a new inode, add the requested entry to the arbo and inform the network
    pub fn register_new_file(&self, entry: FsEntry) -> u64 {
        let ino = self.get_next_inode();

        {
            let mut arbo = self.arbo.lock().expect("arbo lock error");
            arbo.insert(ino, entry.clone());
        }

        self.network_sender
            .send(ToNetworkMessage::BroadcastMessage(message::MessageContent::File(message::File {
                path: entry.get_path().to_path_buf(),
                ino: ino,
            })))
            .expect("mkfile: unable to update modification on the network thread");
        // TODO - if unable to update for some reason, should be passed to the background worker

        ino
    }

    /// remove the requested entry to the arbo and inform the network
    pub fn unregister_file(&self, path: PathBuf) {
        {
            let mut arbo = self.arbo.lock().expect("arbo lock error");
            arbo.retain(|_, entry | *entry.get_path() != path)
        }

        self.network_sender
            .send(ToNetworkMessage::BroadcastMessage(message::MessageContent::Remove(0)))
            .expect("mkfile: unable to update modification on the network thread");
        // TODO - if unable to update for some reason, should be passed to the background worker
    }
}
