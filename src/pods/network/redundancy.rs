use super::network_interface::{get_all_peers_address, NetworkInterface};
use crate::{
    error::{WhError, WhResult},
    network::message::{Address, MessageContent, RedundancyMessage, ToNetworkMessage},
    pods::{
        arbo::{Arbo, FsEntry, InodeId},
        filesystem::fs_interface::FsInterface,
    },
};
use futures_util::future::join_all;
use std::{error, future::Future, sync::Arc};
use tokio::{
    sync::{
        futures,
        mpsc::{unbounded_channel, UnboundedReceiver},
    },
    task::JoinSet,
};

custom_error::custom_error! {pub RedundancyError
    WhError{source: WhError} = "{source}",
    InsufficientHosts = "Redundancy: Not enough nodes to satisfies the target redundancies number.", // warning only
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
                    &peers,
                    &self_addr,
                    ino,
                )
                .await
            }
            RedundancyMessage::CheckIntegrity => {
                check_integrity(&nw_interface, &fs_interface, redundancy, &peers, &self_addr).await
            }
        }
        .inspect_err(|e| log::error!("Redundancy error: {e}"));
    }
}

/// Checks if an inode can have it's redundancy applied :
/// - needs more hosts
/// - the network contains more hosts
/// - this node possesses the file
/// - this node is first on the sorted hosts list (naive approach to avoid many hosts applying the same file)
///
/// Intended for use in the check_intergrity function
fn eligible_to_apply(
    target_redundancy: u64,
    available_peers: usize,
    hosts: &Vec<Address>,
    self_addr: &Address,
) -> bool {
    hosts.clone().sort();
    hosts.len() < target_redundancy as usize
        && available_peers > hosts.len()
        && hosts[0] == *self_addr
}

async fn check_integrity(
    nw_interface: &Arc<NetworkInterface>,
    fs_interface: &Arc<FsInterface>,
    redundancy: u64,
    peers: &Vec<Address>,
    self_addr: &Address,
) -> Result<(), RedundancyError> {
    log::debug!("Checking redundancy integrity");

    let available_peers = peers.len() + 1;

    // Applies redundancy to needed files
    let futures = Arbo::n_read_lock(&nw_interface.arbo, "redundancy: check_integrity")?
        .iter()
        .filter(|(_, inode)| matches!(&inode.entry, FsEntry::File(hosts) if eligible_to_apply(redundancy, available_peers, hosts, self_addr)))
        .map(|(ino, _)| apply_to(nw_interface, fs_interface, redundancy, peers, self_addr, ino.clone()))
        .collect::<Vec<_>>();

    let mut errors: Vec<RedundancyError> = Vec::new();
    // couting for: (ok: enough redundancies, er: errors, ih: still not enough hosts)
    let (ok, er, ih) =
        join_all(futures)
            .await
            .into_iter()
            .fold((0, 0, 0), |(ok, er, ih), status| match status {
                Ok(_) => (ok + 1, er, ih),
                Err(RedundancyError::InsufficientHosts) => (ok, er, ih + 1),
                Err(e) => {
                    errors.push(e);
                    (ok, er + 1, ih)
                }
            });

    log::info!("Redundancy integrity checked for all files. {ok} files updated.");
    if ih > 0 {
        log::warn!("Still {ih} files can't have enough redundancies.");
    }
    if er > 0 {
        log::error!("{er} errors reported !");
        errors.iter().for_each(|e| log::error!("{e}"));
    }
    Ok(())
}

async fn apply_to(
    nw_interface: &Arc<NetworkInterface>,
    fs_interface: &Arc<FsInterface>,
    redundancy: u64,
    peers: &Vec<Address>,
    self_addr: &Address,
    ino: u64,
) -> Result<(), RedundancyError> {
    let file_binary = Arc::new(fs_interface.read_local_file(ino)?);

    let not_enough_hosts: bool;
    let target_redundancy = if (redundancy - 1) as usize > peers.len() {
        not_enough_hosts = true;
        peers.len()
    } else {
        not_enough_hosts = false;
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

    nw_interface.update_hosts(ino, new_hosts)?;
    if not_enough_hosts {
        Err(RedundancyError::InsufficientHosts)
    } else {
        Ok(())
    }
}

/// start download to others concurrently
async fn push_redundancy(
    nw_interface: &Arc<NetworkInterface>,
    all_peers: &Vec<String>,
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
