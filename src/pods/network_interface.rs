use std::{
    io,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use parking_lot::RwLock;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::{
    network::message::{self, ToNetworkMessage},
    providers::whpath::WhPath,
};

use super::{
    disk_manager::DiskManager,
    inode::{Arbo, FsEntry, Inode, InodeId, LOCK_TIMEOUT},
};

pub struct NetworkInterface {
    pub arbo: Arc<RwLock<Arbo>>,
    pub mount_point: WhPath, // TODO - replace by Ludo's unipath
    pub network_sender: UnboundedSender<ToNetworkMessage>,
    pub next_inode: InodeId, // TODO - replace with InodeIndex type
    pub network_airport_handle: JoinHandle<()>,
}

impl NetworkInterface {
    fn get_next_inode(&mut self) -> u64 {
        let available_inode = self.next_inode;
        self.next_inode += 1;

        available_inode
    }

    #[must_use]
    /// Get a new inode, add the requested entry to the arbo and inform the network
    pub fn register_new_file(
        &self,
        inode: Inode,
    ) -> io::Result<u64> {
        let new_inode_id = self.get_next_inode();

        if let Some(mut arbo) = self.arbo.try_read_for(LOCK_TIMEOUT) {
            // REVIEW - should be try_write_for, but testing for science as the compiler didn't say anything
            arbo.add_inode(new_inode_id, inode);
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "mkfile: can't write-lock arbo's RwLock",
            ));
        };

        self.network_sender
            .send(ToNetworkMessage::BroadcastMessage(
                message::MessageContent::File(entry, ino),
            ))
            .expect("mkfile: unable to update modification on the network thread");
        // TODO - if unable to update for some reason, should be passed to the background worker

        ino
    }

    /// remove the requested entry to the arbo and inform the network
    pub fn unregister_file(&self, path: PathBuf) {
        {
            let mut arbo = self.arbo.lock().expect("arbo lock error");
            arbo.retain(|_, entry| *entry.get_path() != path)
        }

        self.network_sender
            .send(ToNetworkMessage::BroadcastMessage(
                message::MessageContent::Remove(0),
            ))
            .expect("mkfile: unable to update modification on the network thread");
        // TODO - if unable to update for some reason, should be passed to the background worker
    }
}
