use std::{io, sync::Arc};

use parking_lot::{Mutex, RwLock};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

use crate::{
    network::message::{self, FromNetworkMessage, MessageContent, ToNetworkMessage},
    providers::whpath::WhPath,
};

use super::{
    arbo::{Arbo, Inode, InodeId, LOCK_TIMEOUT},
    fs_interface::FsInterface,
};

pub struct NetworkInterface {
    pub arbo: Arc<RwLock<Arbo>>,
    pub mount_point: WhPath, // TODO - replace by Ludo's unipath
    pub network_sender: UnboundedSender<ToNetworkMessage>,
    pub next_inode: Mutex<InodeId>, // TODO - replace with InodeIndex type
    pub network_airport_handle: JoinHandle<()>,
}

impl NetworkInterface {
    fn get_next_inode(&self) -> io::Result<u64> {
        let mut next_inode = match self.next_inode.try_lock_for(LOCK_TIMEOUT) {
            Some(lock) => Ok(lock),
            None => Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "get_next_inode: can't lock next_inode",
            )),
        }?;
        let available_inode = *next_inode;
        *next_inode += 1;

        Ok(available_inode)
    }

    #[must_use]
    /// Get a new inode, add the requested entry to the arbo and inform the network
    pub fn register_new_file(&self, inode: Inode) -> io::Result<u64> {
        let new_inode_id = self.get_next_inode()?;

        if let Some(mut arbo) = self.arbo.try_write_for(LOCK_TIMEOUT) {
            arbo.add_inode(new_inode_id, inode.clone())?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "mkfile: can't write-lock arbo's RwLock",
            ));
        };

        // TODO - add myself to hosts

        self.network_sender
            .send(ToNetworkMessage::BroadcastMessage(
                message::MessageContent::Inode(inode, new_inode_id),
            ))
            .expect("mkfile: unable to update modification on the network thread");
        // TODO - if unable to update for some reason, should be passed to the background worker

        Ok(new_inode_id)
    }

    #[must_use]
    /// Get a new inode, add the requested entry to the arbo and inform the network
    pub fn acknowledge_new_file(&self, inode: Inode, id: InodeId) -> io::Result<()> {
        if let Some(mut arbo) = self.arbo.try_write_for(LOCK_TIMEOUT) {
            match arbo.add_inode(id, inode) {
                Ok(()) => (),
                Err(_) => todo!("acknowledge_new_file: file already existing: conflict"), // TODO
            };
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "mkfile: can't write-lock arbo's RwLock",
            ));
        };

        Ok(())
    }

    /// remove the requested entry to the arbo and inform the network
    pub fn unregister_file(&self, id: InodeId) -> io::Result<Inode> {
        let removed_inode: Inode;

        if let Some(mut arbo) = self.arbo.try_write_for(LOCK_TIMEOUT) {
            removed_inode = arbo.remove_inode(id)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "mkfile: can't write-lock arbo's RwLock",
            ));
        };

        self.network_sender
            .send(ToNetworkMessage::BroadcastMessage(
                message::MessageContent::Remove(id),
            ))
            .expect("mkfile: unable to update modification on the network thread");

        // TODO - if unable to update for some reason, should be passed to the background worker

        Ok(removed_inode)
    }

    pub fn acknowledge_unregister_file(&self, id: InodeId) -> io::Result<Inode> {
        if let Some(mut arbo) = self.arbo.try_write_for(LOCK_TIMEOUT) {
            removed_inode = arbo.remove_inode(id)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "mkfile: can't write-lock arbo's RwLock",
            ));
        };

        self.network_sender
            .send(ToNetworkMessage::BroadcastMessage(
                message::MessageContent::Remove(id),
            ))
            .expect("mkfile: unable to update modification on the network thread");

        // TODO - if unable to update for some reason, should be passed to the background worker

        Ok(removed_inode)
    }
}

pub async fn netowrk_airport(
    mut nfa_rx: UnboundedReceiver<FromNetworkMessage>,
    fs_interface: Arc<FsInterface>,
) {
    loop {
        let FromNetworkMessage { origin, content } = match nfa_rx.recv().await {
            Some(message) => message,
            None => continue,
        };

        match content {
            MessageContent::Binary(bin) => {
                println!("peer: {:?}", String::from_utf8(bin).unwrap_or_default());
            }
            MessageContent::Inode(inode, id) => {
                fs_interface.recept_inode(inode, id);
            }
            MessageContent::Remove(ino) => {
                let mut provider = provider.lock().expect("failed to lock mutex");
                provider.recpt_remove(ino);
            }
            MessageContent::Write(ino, data) => {
                // deprecated ?
                let mut provider = provider.lock().expect("failed to lock mutex");
                provider.recpt_write(ino, data);
            }
            MessageContent::Meta(_) => {}
            MessageContent::RequestFile(_) => {}
            MessageContent::RequestFs => {
                let provider = provider.lock().expect("failed to lock mutex");
                provider.send_file_system(origin);
            }
            MessageContent::FileStructure(fs) => {
                let mut provider = provider.lock().expect("failed to lock mutex");
                provider.merge_file_system(fs);
            }
        };
    }
}
