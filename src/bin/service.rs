// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use std::{
    collections::HashMap,
    env,
    error::Error,
    sync::{Arc, RwLock},
};

use futures_util::{lock, StreamExt};
use log::error;
use notify::Watcher;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot, Mutex,
    },
};

use wormhole::{config, data::metadata::MetaData, network::forward::{forward_read_to_sender, forward_receiver_to_write}};
use wormhole::network::peer_ipc::PeerIPC;

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

// async fn publish(pod_path: &std::path::Path, change_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
//     let nw = WormHole::config::Network::read(pod_path.join(".wormhole").join("network.toml"))?;
//     let file = std::fs::read(change_path)?;
//     let change = NetworkMessage::File(File { path: change_path.to_owned(), file});
//     let serialized = bincode::serialize(&change)?;
//     for peer in nw.peers {
//         match tokio::net::TcpStream::connect(&peer).await {
//             Ok(mut connection) => {
//                 if let Err(e) = connection.write(&serialized).await {
//                     error!("sending {:?} to peer {} failed in {}", &change_path, &peer, e);
//                 }
//             },
//             Err(_) => {
//                 error!("peer {} is unavailable", &peer);
//             }
//         }
//     }
//     Ok(())
// }

async fn publish_meta<'a>(
    state: &'a Arc<State>,
    pod_path: &std::path::Path,
    file_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + 'a>> {
    let pods = state.pods.read()?;
    let nw = &pods
        .get(pod_path)
        .ok_or(std::io::Error::other("pod not registered"))?
        .network;
    let file = std::fs::read(file_path)?;
    let change = NetworkMessage::Meta(MetaData::read(file_path)?);
    for peer in &nw.peers {
        let lock = state.peers.read()?;
        if let Some(found) = lock.iter().find(|p| p.address == *peer) {
            found.sender.send(change.clone()).await;
        } else {
            drop(lock);
            let mut lock = state.peers.write()?;
            let peer_ipc = PeerIPC::connect(peer.clone());
            peer_ipc.sender.send(change.clone()).await;
            lock.push(peer_ipc);
        }
    }
    Ok(())
}

// async fn storage_watchdog() -> Result<(), Box<dyn std::error::Error>> {
// =
//     Ok(())
// }

async fn local_watchdog(
    user_tx: UnboundedSender<NetworkMessage>,
    mut peer_rx: UnboundedReceiver<NetworkMessage>,
) {
    let mut stdin = tokio::io::stdin();
    let mut buf = vec![0; 1024];
    loop {
        tokio::select! {
            read = stdin.read(&mut buf) => {
                match read {
                    Err(_) | Ok(0) => { println!("EOF"); break},
                    Ok(n) => {
                        buf.truncate(n);
                        user_tx.send(NetworkMessage::Binary(buf.to_owned())).unwrap();
                    }
                };
            }
            out = peer_rx.recv() => {
                match out.unwrap() {
                    NetworkMessage::File(change) => {
                        println!("peer: {:?}",change);
                    }
                    NetworkMessage::Binary(bin) => {
                        println!("peer: {:?}",String::from_utf8(bin).unwrap_or_default());
                    }
                    NetworkMessage::Meta(_) => todo!(),
                    NetworkMessage::RequestFile(_) => todo!(), };
            }
            // storage = storage_watchdog() => {
            //     match storage {
            //         Ok(_) => (),
            //         Err(_) => (),
            //     }
            // }
        };
    }
}

async fn remote_watchdog(
    server: Server,
    peer_tx: UnboundedSender<NetworkMessage>,
    mut user_rx: UnboundedReceiver<NetworkMessage>,
) {
    while let Ok((stream, _)) = server.listener.accept().await {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");
        let (write, read) = ws_stream.split();
        tokio::join!(
            forward_read_to_sender(read, peer_tx.clone()),
            forward_receiver_to_write(write, &mut user_rx)
        );
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut state = State::default();

    let own_addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let other_addr = env::args()
        .nth(2)
        .unwrap_or("ws://127.0.0.2:8080".to_string());

    let (peer_tx, peer_rx) = mpsc::unbounded_channel();
    let (user_tx, mut user_rx) = mpsc::unbounded_channel();

    tokio::spawn(local_watchdog(user_tx, peer_rx));

    if let Ok((ws_stream, _)) = tokio_tungstenite::connect_async(other_addr).await {
        let (write, read) = ws_stream.split();
        tokio::join!(
            forward_read_to_sender(read, peer_tx),
            forward_receiver_to_write(write, &mut user_rx)
        );
    } else {
        let server = Server::setup(&own_addr).await;
        let _ = tokio::spawn(remote_watchdog(server, peer_tx, user_rx)).await;
    }
}
