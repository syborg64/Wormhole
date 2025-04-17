use std::{io, sync::Arc};

use crate::config::GlobalConfig;
use crate::error::{WhError, WhResult};
#[cfg(target_os = "linux")]
use crate::fuse::fuse_impl::mount_fuse;
use crate::network::message::{
    FileSystemSerialized, FromNetworkMessage, MessageContent, ToNetworkMessage,
};
#[cfg(target_os = "windows")]
use crate::winfsp::winfsp_impl::mount_fsp;
use custom_error::custom_error;
#[cfg(target_os = "linux")]
use fuser;
use log::info;
use parking_lot::RwLock;
use serde::Serialize;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
#[cfg(target_os = "windows")]
use winfsp::host::FileSystemHost;

use crate::network::{message::Address, peer_ipc::PeerIPC, server::Server};

use crate::pods::{
    arbo::{generate_arbo, Arbo},
    disk_manager::DiskManager,
    filesystem::fs_interface::FsInterface,
    network::network_interface::NetworkInterface,
    whpath::WhPath,
};

use super::arbo::{InodeId, ARBO_FILE_FNAME};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Pod {
    name: String,
    network_interface: Arc<NetworkInterface>,
    fs_interface: Arc<FsInterface>,
    mount_point: WhPath,
    peers: Arc<RwLock<Vec<PeerIPC>>>,
    #[cfg(target_os = "linux")]
    fuse_handle: fuser::BackgroundSession,
    #[cfg(target_os = "windows")]
    fsp_host: FileSystemHost<'static>,
    network_airport_handle: Option<JoinHandle<()>>,
    peer_broadcast_handle: Option<JoinHandle<()>>,
    new_peer_handle: Option<JoinHandle<()>>,
}

pub async fn initiate_connection(
    peers_addrs: Vec<Address>,
    server_address: Address,
    tx: &UnboundedSender<FromNetworkMessage>,
    rx: &mut UnboundedReceiver<FromNetworkMessage>,
) -> Option<(FileSystemSerialized, Vec<Address>, PeerIPC)> {
    if peers_addrs.len() >= 1 {
        for first_contact in peers_addrs {
            let first_ipc = PeerIPC::connect(first_contact.to_owned(), tx.clone()).await;

            if let Some(ipc) = first_ipc {
                if let Err(err) = ipc.sender.send((MessageContent::RequestFs, None)) {
                    info!(
                        "Connection with {first_contact} failed: {err}.\n
                        Trying with next know address"
                    );
                    continue;
                }

                loop {
                    match rx.recv().await {
                        Some(FromNetworkMessage {
                            origin: _,
                            content: MessageContent::FsAnswer(fs, mut peers_address),
                        }) => {
                            // remove itself from peers and first_connect because the connection is already existing
                            peers_address.retain(|address| {
                                *address != server_address && *address != first_contact
                            });
                            return Some((fs, peers_address, ipc));
                        }
                        Some(_) => {
                            info!(
                                "First message with {first_contact} failed: His answer is not the FileSystem, corrupted client.\n
                                Trying with next know address"
                            );
                            break;
                        }
                        None => continue,
                    };
                }
            }
        }
        info!("None of the known address answered correctly, starting a FS.")
    }
    None
}

fn register_to_others(peers: &Vec<PeerIPC>, self_address: &Address) -> std::io::Result<()> {
    for peer in peers {
        peer.sender
            .send((MessageContent::Register(self_address.clone()), None))
            .map_err(|err| std::io::Error::new(io::ErrorKind::NotConnected, err))?;
    }
    Ok(())
}

custom_error! {pub PodStopError
    WhError{source: WhError} = "{source}",
    ArboSavingFailed{error_source: String} = @{format!("PodStopError: could not write arbo to disk: {error_source}")},
    PodNotRunning = "No pod with this name was found running.",
    FileNotReadable{file: WhPath, reason: String} = @{format!("PodStopError: could not read file from disk: {file} {reason}")},
    FileNotSent{file: WhPath} = @{format!("PodStopError: no pod was able to receive this file before stopping: {file}")}
}

impl Pod {
    pub async fn new(
        name: String,
        global_config: GlobalConfig,
        mount_point: WhPath,
        server: Arc<Server>,
        server_address: Address,
    ) -> io::Result<Self> {
        let mut global_config = global_config;

        log::info!("mount point {}", mount_point);
        let (mut arbo, next_inode) =
            generate_arbo(&mount_point, &server_address).expect("unable to index folder");
        let (to_network_message_tx, to_network_message_rx) = mpsc::unbounded_channel();
        let (from_network_message_tx, mut from_network_message_rx) = mpsc::unbounded_channel();

        global_config.general.peers.retain(|x| *x != server_address);

        let mut peers = vec![];

        if let Some((fs_serialized, peers_addrs, ipc)) = initiate_connection(
            global_config.general.peers,
            server_address.clone(),
            &from_network_message_tx,
            &mut from_network_message_rx,
        )
        .await
        {
            peers = PeerIPC::peer_startup(peers_addrs, from_network_message_tx.clone()).await;
            peers.push(ipc);
            register_to_others(&peers, &server_address)?;
            arbo.overwrite_self(fs_serialized.fs_index);
        }

        let arbo: Arc<RwLock<Arbo>> = Arc::new(RwLock::new(arbo));

        let network_interface = Arc::new(NetworkInterface::new(
            arbo.clone(),
            mount_point.clone(),
            to_network_message_tx,
            next_inode,
            Arc::new(RwLock::new(peers)),
            server_address,
            global_config.redundancy.number,
        ));

        let disk_manager = DiskManager::new(mount_point.clone())?;
        let fs_interface = Arc::new(FsInterface::new(
            network_interface.clone(),
            disk_manager,
            arbo.clone(),
        ));

        // Start ability to recieve messages
        let network_airport_handle = Some(tokio::spawn(NetworkInterface::network_airport(
            from_network_message_rx,
            fs_interface.clone(),
        )));

        // Start ability to send messages
        let peer_broadcast_handle = Some(tokio::spawn(NetworkInterface::contact_peers(
            network_interface.peers.clone(),
            to_network_message_rx,
        )));

        let new_peer_handle = Some(tokio::spawn(
            NetworkInterface::incoming_connections_watchdog(
                server,
                from_network_message_tx,
                network_interface.peers.clone(),
            ),
        ));

        let peers = network_interface.peers.clone();

        Ok(Self {
            name: name.clone(),
            network_interface,
            fs_interface: fs_interface.clone(),
            mount_point: mount_point.clone(),
            peers,
            #[cfg(target_os = "linux")]
            fuse_handle: mount_fuse(&mount_point, fs_interface.clone())?,
            #[cfg(target_os = "windows")]
            fsp_host: mount_fsp(&mount_point, fs_interface.clone())?,
            network_airport_handle,
            peer_broadcast_handle,
            new_peer_handle,
        })
    }

    /// for a given file, will try to send it to one host, trying each until succes
    fn send_file_to_possible_hosts(
        &self,
        possible_hosts: &Vec<Address>,
        ino: InodeId,
        path: WhPath,
    ) -> Result<(), PodStopError> {
        let file_content = self
            .fs_interface
            .disk
            .read_file_to_end(path.clone())
            .map_err(|e| PodStopError::FileNotReadable {
                file: path.clone(),
                reason: e.to_string(),
            })?;

        for host in possible_hosts {
            let (status_tx, mut status_rx) = tokio::sync::mpsc::unbounded_channel::<WhResult<()>>();

            self.network_interface
                .to_network_message_tx
                .send(ToNetworkMessage::SpecificMessage(
                    (
                        // NOTE - file_content clone is not efficient, but no way to avoid it for now
                        MessageContent::PullAnswer(ino, file_content.clone()),
                        Some(status_tx.clone()),
                    ),
                    vec![host.clone()],
                ))
                .expect("to_network_message_tx closed.");

            if let Ok(()) = status_rx.blocking_recv().unwrap() {
                self.network_interface
                    .to_network_message_tx
                    .send(ToNetworkMessage::BroadcastMessage(
                        MessageContent::RemoveHosts(
                            ino,
                            vec![self.network_interface.self_addr.clone()],
                        ),
                    ))
                    .expect("to_network_message_tx closed.");
                return Ok(());
            }
        }
        Err(PodStopError::FileNotSent { file: path })
    }

    /// Gets every file hosted by this pod only and sends them to other pods
    fn send_files_when_stopping(&self, arbo: &Arbo, peers: Vec<Address>) {
        arbo.files_hosted_only_by(&self.network_interface.self_addr)
            .filter_map(|inode| {
                Some((
                    inode.id,
                    arbo.n_get_path_from_inode_id(inode.id)
                        .map_err(|e| log::error!("Pod::files_to_send_when_stopping(2): {e}"))
                        .ok()?,
                ))
            })
            .for_each(|(id, path)| {
                if let Err(e) = self.send_file_to_possible_hosts(&peers, id, path) {
                    log::warn!("{e}");
                }
            });
    }

    pub fn stop(&self) -> Result<(), PodStopError> {
        // TODO
        // in actual state, all operations (request from network other than just pulling the asked files)
        // made after calling this function but before dropping the pod are undefined behavior.

        // drop(self.fuse_handle); // FIXME - do something like block the filesystem

        let arbo = Arbo::n_read_lock(&self.network_interface.arbo, "Pod::Pod::stop(1)")?;

        let peers: Vec<Address> = self
            .peers
            .read()
            .iter()
            .map(|peer| peer.address.clone())
            .collect();

        self.send_files_when_stopping(&arbo, peers);

        self.network_interface
            .to_network_message_tx
            .send(ToNetworkMessage::BroadcastMessage(
                MessageContent::Disconnect(self.network_interface.self_addr.clone()),
            ))
            .expect("to_network_message_tx closed.");

        let _ = self.fs_interface.disk.remove_file(ARBO_FILE_FNAME.into());
        self.fs_interface
            .disk
            .write_file(
                ARBO_FILE_FNAME.into(),
                &bincode::serialize(&*arbo).expect("can't serialize arbo to bincode"),
                0,
            )
            .map(|_| ())
            .map_err(|e| PodStopError::ArboSavingFailed {
                error_source: e.to_string(),
            })
    }
}
