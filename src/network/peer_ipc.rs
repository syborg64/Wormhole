use futures_util::StreamExt;
use tokio::sync::mpsc;

use crate::network::forward::{forward_read_to_sender2, forward_receiver_to_write2};

use super::message::NetworkMessage;

pub struct PeerIPC {
    pub address: String,
    pub thread: tokio::task::JoinHandle<()>,
    pub sender: mpsc::Sender<NetworkMessage>, // send a message to the peer
    pub receiver: mpsc::Receiver<NetworkMessage>, // receive a message from the peer
}

impl PeerIPC {
    async fn work(address: String, sender: mpsc::Sender<NetworkMessage>, mut receiver: mpsc::Receiver<NetworkMessage>) {
        if let Ok((stream, _)) = tokio_tungstenite::connect_async(&address).await {
            let (write, read) = stream.split();
            tokio::join!(
                forward_read_to_sender2(read, sender),
                forward_receiver_to_write2(write, &mut receiver)
            );
        }
    }

    pub fn connect(address: String) -> Self {
        let (outbound_send, outbound_recv) = mpsc::channel(16);
        let (inbound_send, inbound_recv) = mpsc::channel(16);
        Self {
            thread: tokio::spawn(Self::work(address.clone(), inbound_send, outbound_recv)),
            address,
            sender: outbound_send,
            receiver: inbound_recv,
        }
    }
}
