use super::network_interface::{get_all_peers_address, NetworkInterface};
use crate::{
    error::{WhError, WhResult},
    network::message::{Address, MessageContent, RedundancyMessage, ToNetworkMessage},
    pods::{arbo::InodeId, filesystem::fs_interface::FsInterface},
};
use std::sync::Arc;
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
    task::JoinSet,
};

custom_error::custom_error! {pub RedundancyError
    WhError{source: WhError} = "{source}",
}

/// Redundancy Worker
/// Worker that applies the redundancy to files
pub async fn redundancy_worker(
    mut reception: UnboundedReceiver<RedundancyMessage>,
    nw_interface: Arc<NetworkInterface>,
    fs_interface: Arc<FsInterface>,
    redundancy: u64, // TODO - when updated in conf, send a message to this worker for update
    self_addr: Address, // TODO - Same
) {
    loop {
        let message = match reception.recv().await {
            Some(message) => message,
            None => continue,
        };
        let peers = match get_all_peers_address(&nw_interface.peers) {
            Ok(peers) => peers,
            Err(e) => {
                log::error!(
                    "Redundancy: can't get peers: (ignoring order {:?}) because of: {e}",
                    message
                );
                continue;
            }
        };

        let _ = match message {
            RedundancyMessage::ApplyTo(ino) => {
                apply_to(
                    &nw_interface,
                    &fs_interface,
                    redundancy,
                    peers,
                    &self_addr,
                    ino,
                )
                .await
            }
            RedundancyMessage::CheckIntegrity => {
                todo!()
            }
        }
        .inspect_err(|e| log::error!("Redundancy error: {e}"));
    }
}

async fn check_integrity(
    nw_interface: &Arc<NetworkInterface>,
    fs_interface: &Arc<FsInterface>,
    redundancy: u64,
    peers: Vec<Address>,
    self_addr: &String,
    ino: u64,
) -> Result<(), RedundancyError> {
    todo!()
}

async fn apply_to(
    nw_interface: &Arc<NetworkInterface>,
    fs_interface: &Arc<FsInterface>,
    redundancy: u64,
    peers: Vec<Address>,
    self_addr: &String,
    ino: u64,
) -> Result<(), RedundancyError> {
    let file_binary = match fs_interface.read_local_file(ino) {
        Ok(bin) => bin,
        Err(e) => {
            return Err(RedundancyError::WhError { source: e });
        }
    };
    let file_binary = Arc::new(file_binary);

    let target_redundancy = if (redundancy - 1) as usize > peers.len() {
        log::warn!("Redundancy: Not enough nodes to satisfies the target redundancies number.");
        peers.len()
    } else {
        (redundancy - 1) as usize
    };

    let new_hosts = push_redundancy(
        nw_interface,
        peers,
        ino,
        file_binary,
        target_redundancy,
        self_addr.clone(),
    )
    .await;

    Ok(nw_interface.update_hosts(ino, new_hosts)?)
}

/// start download to others concurrently
async fn push_redundancy(
    nw_interface: &Arc<NetworkInterface>,
    all_peers: Vec<String>,
    ino: InodeId,
    file_binary: Arc<Vec<u8>>,
    target_redundancy: usize,
    self_addr: Address,
) -> Vec<Address> {
    let mut success_hosts: Vec<Address> = vec![self_addr];
    let mut set: JoinSet<WhResult<Address>> = JoinSet::new();

    for i in 0..target_redundancy {
        let nwi_clone = Arc::clone(nw_interface);
        //TODO cloning the whole file content in ram to send it to many hosts is terrible :
        let bin_clone = file_binary.clone();
        let addr = all_peers[i].clone();

        set.spawn(async move { nwi_clone.send_file_redundancy(ino, bin_clone, addr).await });
    }

    // check for success and try next hosts if failure
    let mut current_try = target_redundancy;
    loop {
        match set.join_next().await {
            None => break,
            Some(Err(e)) => {
                log::error!("redundancy_worker: error in thread pool: {e}");
                break;
            }
            Some(Ok(Ok(host))) => success_hosts.push(host),
            Some(Ok(Err(crate::error::WhError::NetworkDied { called_from: _ }))) => {
                log::warn!("Redundancy: NetworkDied on some host. Trying next...");
                if current_try >= all_peers.len() {
                    log::error!("Redundancy: Not enough answering hosts to apply redundancy.");
                    break;
                }
                let nwi_clone = Arc::clone(nw_interface);
                let bin_clone = file_binary.clone();
                //TODO cloning the whole file content in ram to send it to many hosts is terrible
                let addr = all_peers[current_try].clone();

                set.spawn(
                    async move { nwi_clone.send_file_redundancy(ino, bin_clone, addr).await },
                );
                current_try += 1;
            }
            Some(Ok(Err(e))) => {
                log::error!("Redundancy: unknown error when applying redundancy: {e}");
                break;
            }
        }
    }
    success_hosts
}

impl NetworkInterface {
    pub async fn send_file_redundancy(
        &self,
        inode: InodeId,
        data: Arc<Vec<u8>>,
        to: Address,
    ) -> WhResult<Address> {
        let (status_tx, mut status_rx) = unbounded_channel();

        self.to_network_message_tx
            .send(ToNetworkMessage::SpecificMessage(
                (MessageContent::RedundancyFile(inode, data), Some(status_tx)),
                vec![to.clone()],
            ))
            .expect("send_file: unable to update modification on the network thread");

        status_rx
            .recv()
            .await
            .unwrap_or(Err(WhError::NetworkDied {
                called_from: "network_interface::send_file_redundancy".to_owned(),
            }))
            .map(|()| to)
    }
}
