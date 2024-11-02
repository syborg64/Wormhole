// In rust we code
// In code we trust
// AgarthaSoftware - 2024

/**DOC
 * Important variables to know :
 * nfa_rx - nfa_tx
 *  Use nfa_tx to send a file related message to the newtork_file_actions function
 *
 * Important functions to know :
 *
 * local_cli_watchdog
 *  this is the handle linked to the terminal, that will terminate the
 *  program if CTRL-D
 *
 * newtork_file_actions
 *  reads a message (supposely emitted by a peer) related to files actions
 *  and execute instructions on the disk
 */
use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex, RwLock},
};

use futures_util::{stream::select_all, StreamExt};
use tokio::{
    io::AsyncReadExt,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};

use tokio_stream::wrappers::ReceiverStream;
use wormhole::{
    config,
    data::metadata::MetaData,
    network::forward::{forward_read_to_sender, forward_receiver_to_write},
    providers::Provider,
};
use wormhole::{fuse::fuse_impl::mount_fuse, network::peer_ipc::PeerIPC};

use wormhole::network::{message::NetworkMessage, server::Server};

struct Pod {
    network: config::Network,
    directory: Arc<std::fs::DirEntry>,
    // fuser: !,
}

#[derive(Default)]
struct State {
    pub peers: RwLock<Vec<PeerIPC>>,
    pub pods: RwLock<HashMap<std::path::PathBuf, Pod>>,
}

// async fn publish_meta<'a>(
//     state: &'a Arc<State>,
//     pod_path: &std::path::Path,
//     file_path: &std::path::Path,
// ) -> Result<(), Box<dyn std::error::Error + 'a>> {
//     let pods = state.pods.read()?;
//     let nw = &pods
//         .get(pod_path)
//         .ok_or(std::io::Error::other("pod not registered"))?
//         .network;
//     let file = std::fs::read(file_path)?;
//     let change = NetworkMessage::Meta(MetaData::read(file_path)?);
//     for peer in &nw.peers {
//         let lock = state.peers.read()?;
//         if let Some(found) = lock.iter().find(|p| p.address == *peer) {
//             found.sender.send(change.clone()).await;
//         } else {
//             drop(lock);
//             let mut lock = state.peers.write()?;
//             let peer_ipc = PeerIPC::connect(peer.clone());
//             peer_ipc.sender.send(change.clone()).await;
//             lock.push(peer_ipc);
//         }
//     }
//     Ok(())
// }

async fn local_cli_watchdog() {
    let mut stdin = tokio::io::stdin();
    let mut buf = vec![0; 1024];

    loop {
        let read = stdin.read(&mut buf).await;

        // NOTE -  on ctrl-D -> quit
        match read {
            Err(_) | Ok(0) => {
                println!("Quiting!");
                break;
            }
            _ => (),
        };
    }
}

/**DOC
 * reads a message (supposely emitted by a peer) related to files actions
 * and execute instructions on the disk
 *
 * params:
 *  @nfa_rx: reception for file related messages
 *  @provider: fuse instance
*/
async fn network_file_actions(
    mut nfa_rx: UnboundedReceiver<NetworkMessage>,
    provider: Arc<Mutex<Provider>>,
) {
    loop {
        match nfa_rx.recv().await {
            Some(NetworkMessage::Binary(bin)) => {
                println!("peer: {:?}", String::from_utf8(bin).unwrap_or_default());
            }
            Some(NetworkMessage::NewFolder(folder)) => {
                println!("peer: NEW FOLDER");
                let mut provider = provider.lock().expect("failed to lock mutex");
                provider.new_folder(folder.ino, folder.path);
            }
            Some(NetworkMessage::File(file)) => {
                println!("peer: NEW FILE");
                let mut provider = provider.lock().expect("failed to lock mutex");
                provider.new_file(file.ino, file.path);
            }
            Some(NetworkMessage::Remove(ino)) => {
                println!("peer: REMOVE");
                let mut provider = provider.lock().expect("failed to lock mutex");
                provider.recpt_remove(ino);
            }
            Some(NetworkMessage::Write(ino, data)) => {
                println!("peer: WRITE");
                let mut provider = provider.lock().expect("failed to lock mutex");
                provider.recpt_write(ino, data);
            }
            Some(NetworkMessage::Meta(_)) => {
                println!("peer: META");
            }
            Some(NetworkMessage::RequestFile(_)) => {
                println!("peer: REQUEST FILE");
            }
            None => {
                () //REVIEW - Is it ok to loop every time ? the recv should wait or throw None every time ?
            }
        };
    }
}

async fn incoming_connections_watchdog(
    server: Server,
    nfa_tx: UnboundedSender<NetworkMessage>,
    existing_peers: Arc<Mutex<Vec<PeerIPC>>>,
) {
    while let Ok((stream, _)) = server.listener.accept().await {
        println!("connecting new client");
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");
        let addr = ws_stream.get_ref().peer_addr().unwrap().to_string();
        let (write, read) = ws_stream.split();
        let new_peer = PeerIPC::connect_from_incomming(addr, nfa_tx.clone(), write, read);
        {
            existing_peers.lock().unwrap().push(new_peer);
        }
        println!("new client connected");
        // tokio::join!(
        //     forward_read_to_sender(read, nfa_tx.clone()),
        //     forward_receiver_to_write(write, &mut user_rx)
        // );
    }
}

// start connexions to peers
async fn peer_startup(
    peers_ip_list: Vec<String>,
    nfa_tx: UnboundedSender<NetworkMessage>,
) -> Vec<PeerIPC> {
    peers_ip_list
        .into_iter()
        .map(|ip| PeerIPC::connect(ip, nfa_tx.clone()))
        .collect()
}

// wait for message from peers and inform the file manager via nfa_tx
// async fn all_peers_reception(peers_list: Vec<PeerIPC>, nfa_tx: UnboundedSender<NetworkMessage>) {
//     let receptors: Vec<ReceiverStream<NetworkMessage>> = peers_list
//         .into_iter()
//         .map(|peer| ReceiverStream::new(peer.receiver))
//         .collect();
//     let mut stream = select_all(receptors);

//     while let Some(msg) = stream.next().await {
//         nfa_tx.send(msg).unwrap();
//     }
// }

// use futures_util::FutureExt;
// pub async fn select_from_peers(peers: &[&PeerIPC]) -> Option<(usize, NetworkMessage)> {
//     let mut futures = vec![];

//     for (index, peer) in peers.iter().enumerate() {
//         let future = peer.receiver.recv();
//         futures.push(future.boxed());
//     }

//     select_all(futures).await.map(|(result, _, _)| result)
// }

// async fn all_peers_reception2(
//     peers_list: &mut Vec<PeerIPC>,
//     nfa_tx: UnboundedSender<NetworkMessage>,
// ) {
//     let recv_futures: Vec<tokio::task::JoinHandle<Option<NetworkMessage>>> = peers_list
//         .iter_mut()
//         .map(|peer| tokio::spawn(peer.receiver.recv()))
//         .collect();
// }

// receive a message on user_rx and broadcast it to all peers
async fn all_peers_broadcast(
    peers_list: Arc<Mutex<Vec<PeerIPC>>>,
    mut user_rx: UnboundedReceiver<NetworkMessage>,
) {
    //generating peers senders
    let peer_tx: Vec<(UnboundedSender<NetworkMessage>, String)> = peers_list
        .lock()
        .unwrap()
        .iter()
        .map(|peer| (peer.sender.clone(), peer.address.clone()))
        .collect();

    // on message reception, broadcast it to all peers senders
    while let Some(message) = user_rx.recv().await {
        peer_tx.iter().for_each(|peer| {
            peer.0
                .send(message.clone())
                .expect(&format!("failed to send message to peer {}", peer.1))
        });
    }
}

async fn remote_watchdog(
    own_addr: String,
    other_addr: String,
    nfa_tx: UnboundedSender<NetworkMessage>,
    mut user_rx: UnboundedReceiver<NetworkMessage>,
) {
    if let Ok((ws_stream, _)) = tokio_tungstenite::connect_async(other_addr).await {
        let (write, read) = ws_stream.split();

        tokio::join!(
            forward_read_to_sender(read, nfa_tx),
            forward_receiver_to_write(write, &mut user_rx)
        );
    } else {
        let server = Server::setup(&own_addr).await;

        //server_watchdog(server, nfa_tx, user_rx).await;
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // DOC - arguments: own_address other_addr1 other_addr2 mount_to source
    let own_addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let other_addr1 = env::args()
        .nth(2)
        .unwrap_or("ws://127.0.0.2:8080".to_string());
    let other_addr2 = env::args()
        .nth(3)
        .unwrap_or("ws://127.0.0.3:8080".to_string());
    let mount = env::args().nth(4).unwrap_or("./virtual/".to_string());
    let source = env::args().nth(5).unwrap_or("./original/".to_string());

    println!("own address: {}", own_addr);
    println!("peer1 address: {}", other_addr1);
    println!("peer2 address: {}", other_addr2);

    println!("\nstarting");

    let (nfa_tx, nfa_rx) = mpsc::unbounded_channel();
    let (user_tx, user_rx) = mpsc::unbounded_channel();
    let (_session, provider) = mount_fuse(&source, &mount, user_tx.clone());

    let local_cli_handle = tokio::spawn(local_cli_watchdog());
    let nfa_handle = tokio::spawn(network_file_actions(nfa_rx, provider));
    let server = Server::setup(&own_addr).await;

    let peers = peer_startup(vec![other_addr1, other_addr2], nfa_tx.clone()).await;
    let peers: Arc<Mutex<Vec<PeerIPC>>> = Arc::new(Mutex::new(peers));

    let new_conn_handle = tokio::spawn(incoming_connections_watchdog(
        server,
        nfa_tx.clone(),
        peers.clone(),
    ));

    let peers_broadcast_handle = tokio::spawn(all_peers_broadcast(peers.clone(), user_rx));
    // let remote_reception = tokio::spawn(all_peers_reception(connected_peers, nfa_tx));

    println!("started");
    local_cli_handle.await.unwrap(); // keeps the main process alive until interruption from this watchdog;
    println!("stopping");
    new_conn_handle.abort();
    peers.lock().unwrap().iter().for_each(|peer| {
        peer.thread.abort();
    });
    nfa_handle.abort();
    peers_broadcast_handle.abort();
    println!("stopped");
}
