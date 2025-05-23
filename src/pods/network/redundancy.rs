use super::network_interface::{get_all_peers_address, NetworkInterface};
use crate::{
    error::WhResult,
    network::message::{Address, RedundancyMessage, ToNetworkMessage},
    pods::{arbo::InodeId, filesystem::fs_interface::FsInterface},
};
use futures_util::task;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinSet,
};

type JobTargets = HashMap<Address, bool>;
type JobStack = HashMap<InodeId, (u64, JobTargets)>;

// SECTION Job ending
fn check_for_job_success(targets: &JobTargets) -> bool {
    targets.values().fold(true, |acc, target| acc && *target)
}

fn on_success(
    sender: &UnboundedSender<ToNetworkMessage>,
    nw_interface: &Arc<NetworkInterface>,
    ino: u64,
    targets: &HashMap<String, bool>,
) -> bool {
    if check_for_job_success(&targets) {
        sender
            .send(ToNetworkMessage::BroadcastMessage(
                crate::network::message::MessageContent::EditHosts(
                    ino,
                    targets.keys().cloned().collect(),
                ),
            ))
            .expect("redundancy_worker: unable to update modification on the network thread");
        nw_interface
            .acknowledge_hosts_edition(ino, targets.keys().cloned().collect())
            .expect("redundancy_worker: can't edit local hosts");
        return true;
    } else {
        return false;
    }
}
// !SECTION Job ending

// SECTION Job starting
fn choose_hosts(redundancy_nb: u64, all_peers: Vec<Address>) -> Vec<Address> {
    let possible_hosts: Vec<Address> = all_peers
        .into_iter()
        .take(redundancy_nb as usize)
        .collect::<Vec<Address>>();

    if possible_hosts.len() < redundancy_nb as usize {
        log::warn!("redundancy needs enough hosts");
    }
    possible_hosts
}

fn create_job_targets(targets: Vec<Address>) -> JobTargets {
    HashMap::from_iter(targets.into_iter().map(|target| (target, false)))
}
// !SECTION Job starting

/// Redundancy Worker
/// Worker that applies the redundancy to files
pub async fn redundancy_worker(
    mut reception: UnboundedReceiver<RedundancyMessage>,
    sender: UnboundedSender<ToNetworkMessage>,
    nw_interface: Arc<NetworkInterface>,
    fs_interface: Arc<FsInterface>,
) {
    let mut stack_job_id: u64 = 0;
    let mut stack: JobStack = HashMap::new();

    loop {
        let message = match reception.recv().await {
            Some(message) => message,
            None => continue,
        };

        match message {
            RedundancyMessage::ApplyTo(ino) => match get_all_peers_address(&nw_interface.peers) {
                // TODO read the file only once (and not once per host)
                Ok(all_peers) => {
                    // let chosen_hosts = choose_hosts(nw_interface.redundancy, all_peers);
                    let mut success = 0;

                    let target_redundancy = if nw_interface.redundancy as usize > all_peers.len() {
                        log::warn!("Redundancy: Not enough nodes to satisfies the target redundancies number.");
                        all_peers.len()
                    } else {
                        nw_interface.redundancy as usize
                    };

                    // start download to others concurrently
                    let mut set: JoinSet<WhResult<()>> = JoinSet::new();
                    for i in 0..target_redundancy {
                        let fs_clone = Arc::clone(&fs_interface);
                        let addr = all_peers[i].clone();

                        set.spawn(async move {
                            fs_clone.send_file_redundancy(ino, addr, stack_job_id).await
                        });
                    }

                    // check for success and try next hosts if failure
                    let mut current_try = target_redundancy;
                    while success < target_redundancy {
                        match set.join_next().await {
                            None => break,
                            Some(Err(e)) => {
                                log::error!("redundancy_worker: error in thread pool: {e}");
                                break;
                            }
                            Some(Ok(Ok(()))) => success += 1,
                            Some(Ok(Err(crate::error::WhError::NetworkDied {
                                called_from: _,
                            }))) => {
                                log::warn!("Redundancy: NetworkDied on some host. Trying next...");
                                if current_try >= all_peers.len() {
                                    log::error!("Redundancy: Not enough answering hosts to apply redundancy.");
                                    break;
                                }
                                let fs_clone = Arc::clone(&fs_interface);
                                let addr = all_peers[current_try].clone();

                                set.spawn(async move {
                                    fs_clone.send_file_redundancy(ino, addr, stack_job_id).await
                                });
                                current_try += 1;
                            }
                            Some(Ok(Err(e))) => {
                                log::error!(
                                    "Redundancy: unknown error when applying redundancy: {e}"
                                );
                                break;
                            }
                        }
                    }
                    set.join_all().await;

                    stack_job_id += 1;
                }
                Err(e) => {
                    log::error!("Redundancy: can't add job for {}. Error: {}", ino, e);
                }
            },
            RedundancyMessage::ReceivedBy(ino, address, id) => {
                if let Some((job_id, targets)) = stack.get_mut(&ino) {
                    if *job_id != id {
                        log::warn!("Redundancy: received answer for a non-existant job (probably outdated)");
                        continue;
                    }
                    if let Some(target_status) = targets.get_mut(&address) {
                        *target_status = true;
                        if on_success(&sender, &nw_interface, ino, targets) {
                            stack.remove(&ino);
                        }
                    } else {
                        log::error!("Redundancy: received by not targeted host");
                    }
                } else {
                    log::error!(
                        "Redundancy: received answer for a non-existant job (no job for this ino)"
                    )
                }
            }
        }
    }
}
