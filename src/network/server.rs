use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::{
    net::{unix::SocketAddr, TcpListener},
    sync::mpsc::UnboundedReceiver,
};

use super::message::ToNetworkMessage;

pub type Tx = UnboundedReceiver<ToNetworkMessage>;
pub type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

pub struct Server {
    pub listener: TcpListener,
    pub state: PeerMap,
}

impl Server {
    pub async fn setup(addr: &str) -> Server {
        Server {
            listener: TcpListener::bind(addr).await.expect("Failed to bind"),
            state: PeerMap::new(Mutex::new(HashMap::new())),
        }
    }
}
