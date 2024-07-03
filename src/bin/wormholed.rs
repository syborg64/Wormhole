use std::env;

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{error, info};
use notify::Watcher;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::broadcast::{Receiver, Sender},
};
use WormHole::{config::types::Config, INSTANCE_PATH};

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tokio::sync::broadcast;

use tokio::net::TcpListener;
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};

type Tx = Receiver<WormMessage>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
enum WormMessage {
    Change(Change),
    Binary(Vec<u8>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Change {
    pub path: std::path::PathBuf,
    pub file: Vec<u8>,
}

// async fn publish(pod_path: &std::path::Path, change_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
//     let nw = WormHole::config::Network::read(pod_path.join(".wormhole").join("network.toml"))?;
//     let file = std::fs::read(change_path)?;
//     let change = WormMessage::Change(Change { path: change_path.to_owned(), file});
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

async fn local_watchdog(tx: Sender<WormMessage>, mut user_rx: Receiver<WormMessage>,  mut peer_rx: Receiver<WormMessage>) {
    let mut stdin = tokio::io::stdin();
    let mut buf = vec![0; 1024];
    loop {
        tokio::select! {
            read = stdin.read(&mut buf) => {
                match read {
                    Err(_) | Ok(0) => { println!("EOF"); break},
                    Ok(n) => {
                        buf.truncate(n);
                        tx.send(WormMessage::Binary(buf.to_owned())).unwrap();
                    }
                };
            }
            out = user_rx.recv() => {
                match out.unwrap() {
                    WormMessage::Change(change) => {
                        println!("user: {:?}", change);
                    }
                    WormMessage::Binary(bin) => {
                        println!("user: {:?}", String::from_utf8(bin).unwrap_or("".to_string()));
                    }
                };
            }
            out = peer_rx.recv() => {
                match out.unwrap() {
                    WormMessage::Change(change) => {
                        println!("peer: {:?}", change);
                    }
                    WormMessage::Binary(bin) => {
                        println!("peer: {:?}", String::from_utf8(bin).unwrap_or("".to_string()));
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

async fn fwd_messages_to_channel(
    mut write: SplitSink<WebSocketStream<TcpStream>, Message>,
    mut user_rx: Receiver<WormMessage>,
) {
    while let Ok(message) = user_rx.recv().await {
        let serialized = bincode::serialize(&message).unwrap();
        write.send(Message::binary(serialized)).await.unwrap();
    }
}

async fn fwd_snd(mut read: SplitStream<WebSocketStream<TcpStream>>, peer_tx: Sender<WormMessage>) {
    while let Ok(Message::Binary(message)) = read.next().await.unwrap() {
        let deserialized = bincode::deserialize(&message).unwrap();
        peer_tx.send(deserialized).unwrap();
    }
}

async fn fwd_messages_to_channel2(
    mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    mut user_rx: Receiver<WormMessage>,
) {
    while let Ok(message) = user_rx.recv().await {
        let serialized = bincode::serialize(&message).unwrap();
        write.send(Message::binary(serialized)).await.unwrap();
    }
}

async fn fwd_snd2(mut read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, peer_tx: Sender<WormMessage>) {
    while let Ok(Message::Binary(message)) = read.next().await.unwrap() {
        let deserialized = bincode::deserialize(&message).unwrap();
        peer_tx.send(deserialized).unwrap();
    }
}

async fn remote_watchdog(server: Server, peer_tx: Sender<WormMessage>, user_rx: Receiver<WormMessage>) {
    while let Ok((stream, _)) = server.listener.accept().await {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");
        let (write, read) = ws_stream.split();
        tokio::join!(
            fwd_snd(read, peer_tx.clone()),
            fwd_messages_to_channel(write, user_rx.resubscribe())
        );
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let own_addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let other_addr = env::args().nth(2).unwrap_or("ws://127.0.0.2:8080".to_string());

    let (peer_tx, peer_rx) = broadcast::channel::<WormMessage>(16);
    let (user_tx, user_rx) = broadcast::channel::<WormMessage>(16);

    tokio::spawn(local_watchdog(user_tx.clone(), user_rx.resubscribe(), peer_rx.resubscribe()));

    if let Ok((ws_stream, _)) = tokio_tungstenite::connect_async(other_addr).await {
        let (write, read) = ws_stream.split();
        tokio::join!(
            fwd_snd2(read, peer_tx.clone()),
            fwd_messages_to_channel2(write, user_rx.resubscribe())
        );
    } else {
        let server = setup_server(&own_addr).await;
        let _ = tokio::spawn(remote_watchdog(server, peer_tx, user_rx)).await;
    }


    // while let Ok(message) = user_rx.recv().await {
    //     info!("{}", message);
    // }
}
