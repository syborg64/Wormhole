// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use std::{env, fmt::Debug};

use futures_util::{stream::SplitStream, Sink, SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    io::AsyncReadExt,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;

pub type Tx = UnboundedReceiver<NetworkMessage>;
pub type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetworkMessage {
    Change(Change),
    Binary(Vec<u8>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Change {
    pub path: std::path::PathBuf,
    pub file: Vec<u8>,
}

// async fn publish(pod_path: &std::path::Path, change_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
//     let nw = WormHole::config::Network::read(pod_path.join(".wormhole").join("network.toml"))?;
//     let file = std::fs::read(change_path)?;
//     let change = NetworkMessage::Change(Change { path: change_path.to_owned(), file});
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

// async fn fuse_watchdog() -> Result<(), Box<dyn std::error::Error>> {
//     let pods_directory = std::path::Path::new(INSTANCE_PATH).join("pods");
//     let folder = std::fs::read_dir(&pods_directory)?;

//     let mut watcher = notify::recommended_watcher(|res| {
//         match res {
//            Ok(event) => {
//             println!("event: {:?}", event)
//             match event.kind {
//                 notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
//                     event.paths.iter().map(|path| publish(path));
//                 }
//                 _ => todo!(),
//             }
//            },
//            Err(e) => println!("watch error: {:?}", e),
//         }
//     })?;

//     for pod in folder {
//         match pod {
//             Ok(pod) => {
//                 let link = pods_directory.join(pod.file_name());
//                 let real_path = std::fs::canonicalize(&link)?;
//                 watcher.watch(&real_path, notify::RecursiveMode::Recursive)?;
//             },
//             Err(_) => ()
//         }
//     }

//     watcher.
//     // Add a path to be watched. All files and directories at that path and
//     // below will be monitored for changes.
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
                    NetworkMessage::Change(change) => {
                        println!("peer: {:?}", change);
                    }
                    NetworkMessage::Binary(bin) => {
                        println!("peer: {:?}", String::from_utf8(bin).unwrap_or_default());
                    }
                };
            }
            // fuse = fuse_watchdog() => {
            //     match fuse {
            //         Ok(_) => (),
            //         Err(_) => (),
            //     }
            // }
        };
    }
}

pub struct Server {
    pub listener: TcpListener,
    pub state: PeerMap,
}

async fn setup_server(addr: &str) -> Server {
    Server {
        listener: TcpListener::bind(addr).await.expect("Failed to bind"),
        state: PeerMap::new(Mutex::new(HashMap::new())),
    }
}

async fn forward_reciver_to_write<T>(mut write: T, rx: &mut UnboundedReceiver<NetworkMessage>)
where
    T: Sink<Message> + Unpin,
    <T as Sink<Message>>::Error: Debug,
{
    while let Some(message) = rx.recv().await {
        let serialized = bincode::serialize(&message).unwrap();
        write.send(Message::binary(serialized)).await.unwrap();
    }
}

async fn forward_read_to_sender<
    T: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>,
>(
    mut read: SplitStream<T>,
    tx: UnboundedSender<NetworkMessage>,
) {
    while let Ok(Message::Binary(message)) = read.next().await.unwrap() {
        let deserialized = bincode::deserialize(&message).unwrap();
        tx.send(deserialized).unwrap();
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
            forward_reciver_to_write(write, &mut user_rx)
        );
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

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
            forward_reciver_to_write(write, &mut user_rx)
        );
    } else {
        let server = setup_server(&own_addr).await;
        let _ = tokio::spawn(remote_watchdog(server, peer_tx, user_rx)).await;
    }
}
