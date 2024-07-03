use std::env;

use futures_util::StreamExt;
use log::{error, info};
use notify::Watcher;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
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
use tokio_tungstenite::tungstenite::protocol::Message;

type Tx = Receiver<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
enum WormMessage {
    Change(Change),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Change {
    pub path: std::path::PathBuf,
    pub file: Vec<u8>,
}

async fn publish(pod_path: &std::path::Path, change_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let nw = WormHole::config::Network::read(pod_path.join(".wormhole").join("network.toml"))?;
    let file = std::fs::read(change_path)?;
    let change = WormMessage::Change(Change { path: change_path.to_owned(), file});
    let serialized = bincode::serialize(&change)?;
    for peer in nw.peers {
        match tokio::net::TcpStream::connect(&peer).await {
            Ok(mut connection) => {
                if let Err(e) = connection.write(&serialized).await {
                    error!("sending {:?} to peer {} failed in {}", &change_path, &peer, e);
                }
            },
            Err(_) => {
                error!("peer {} is unavailable", &peer);
            }
        }
    }
    Ok(())
}

async fn fuse_watchdog() -> Result<(), Box<dyn std::error::Error>> {
    let pods_directory = std::path::Path::new(INSTANCE_PATH).join("pods");
    let folder = std::fs::read_dir(&pods_directory)?;

    let mut watcher = notify::recommended_watcher(|res| {
        match res {
           Ok(event) => println!("event: {:?}", event),
           Err(e) => println!("watch error: {:?}", e),
        }
    })?;

    for pod in folder {
        match pod {
            Ok(pod) => {
                let link = pods_directory.join(pod.file_name());
                let real_path = std::fs::canonicalize(&link)?;
                watcher.watch(&real_path, notify::RecursiveMode::Recursive)?;
            },
            Err(_) => ()
        }
    }


    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    Ok(())
}

async fn local_watchdog(tx: Sender<Message>) {
    let mut stdin = tokio::io::stdin();
    let mut buf = vec![0; 1024];
    loop {
        tokio::select! {
            read = stdin.read(&mut buf) => {
                match read {
                    Err(_) | Ok(0) => break,
                    Ok(n) => {
                        buf.truncate(n);
                        tx.send(Message::binary(buf.clone())).unwrap();
                    }
                };
            }
            fuse = fuse_watchdog() => {
                match fuse {
                    Ok(_) => (),
                    Err(_) => (),
                }
            }
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

async fn remote_watchdog(mut server: Server, user_rx: Receiver<Message>) {
    tokio::select! {
            Ok((stream, addr)) = server.listener.accept() =>  {
            let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("Error during the websocket handshake occurred");
            let (outgoing, incoming) = ws_stream.split();
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let own_addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let other_addr = env::args().nth(2).unwrap_or("127.0.0.2:8080".to_string());

    let (user_tx, mut user_rx) = broadcast::channel(16);

    tokio::spawn(local_watchdog(user_tx));

    let server = setup_server(&own_addr).await;
    tokio::spawn(remote_watchdog(server, user_rx.resubscribe()));

    // while let Ok(message) = user_rx.recv().await {
    //     info!("{}", message);
    // }
}
