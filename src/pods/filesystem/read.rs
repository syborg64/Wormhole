use std::io;

use custom_error::custom_error;

use crate::network::message::{MessageContent, ToNetworkMessage};
use crate::pods::arbo::{Arbo, FsEntry};
use crate::pods::network::network_interface::{Callback, NetworkInterface};
use crate::{error::WhError, pods::arbo::InodeId};

use super::fs_interface::FsInterface;

custom_error! {
    /// Error describing the read syscall
    pub ReadError
    WhError{source: WhError} = "{source}",
    CantPull = "Unable to pull file"
}

impl NetworkInterface {
    // REVIEW - recheck and simplify this if possible
    pub fn pull_file_sync(&self, file: InodeId) -> io::Result<Option<Callback>> {
        let hosts = {
            let arbo = Arbo::read_lock(&self.arbo, "pull_file")?;
            if let FsEntry::File(hosts) = &arbo.get_inode(file)?.entry {
                hosts.clone()
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "pull_file: can't pull a folder",
                ));
            }
        };

        if hosts.len() == 0 {
            log::error!("No hosts hold the file");
            return Err(io::ErrorKind::InvalidData.into());
        }

        if hosts.contains(&self.self_addr) {
            // if the asked file is already on disk
            Ok(None)
        } else {
            let callback = self.callbacks.create(Callback::Pull(file))?;
            let (status_tx, mut status_rx) = tokio::sync::mpsc::unbounded_channel::<WhResult<()>>();

            // will try to pull on all redundancies until success
            for host in hosts {
                // trying on host `pull_from`
                self.to_network_message_tx
                    .send(ToNetworkMessage::SpecificMessage(
                        (
                            MessageContent::RequestFile(file, self.self_addr.clone()),
                            Some(status_tx.clone()),
                        ),
                        vec![host.clone()], // NOTE - naive choice for now
                    ))
                    .expect("pull_file: unable to request on the network thread");

                // processing status
                match status_rx
                    .blocking_recv()
                    .expect("pull_file: unable to get status from the network thread")
                {
                    Ok(()) => return Ok(Some(callback)),
                    Err(_) => continue,
                }
            }
            let _ = self.callbacks.resolve(callback, true);
            log::error!("No host is currently able to send the file\nFile: {file}");
            return Err(io::ErrorKind::NotConnected.into());
        }
    }
}

impl FsInterface {
    pub fn read_file(&self, file: InodeId, offset: u64, len: u64) -> Result<Vec<u8>, ReadError> {
        //let cb = self.network_interface.pull_file_sync(file);

        // let status = match cb {
        //     None => true,
        //     Some(call) => self.network_interface.callbacks.wait_for(call)?,
        // };

        // if !status {
        //     return Err(ReadError::CantPull);
        // }

        // self.disk.read_file(
        //     Arbo::n_read_lock(&self.arbo, "read_file")?.n_get_path_from_inode_id(file)?,
        //     offset,
        //     len,
        // )
        return Ok(vec![]);
    }
}
