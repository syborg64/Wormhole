use futures_util::StreamExt;
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::network::forward::{forward_read_to_sender, forward_receiver_to_write};

use super::message::NetworkMessage;

pub struct PeerIPC {
    pub address: String,
    pub thread: tokio::task::JoinHandle<()>,
    pub sender: mpsc::UnboundedSender<NetworkMessage>, // send a message to the peer
                                                       // pub receiver: mpsc::Receiver<NetworkMessage>, // receive a message from the peer
}

impl PeerIPC {
    async fn work(
        address: String,
        sender: mpsc::UnboundedSender<NetworkMessage>,
        mut receiver: mpsc::UnboundedReceiver<NetworkMessage>,
    ) {
        if let Ok((stream, _)) = tokio_tungstenite::connect_async(&address).await {
            let (write, read) = stream.split();
            tokio::join!(
                forward_read_to_sender(read, sender),
                forward_receiver_to_write(write, &mut receiver)
            );
        }
    }

    pub fn connect(address: String, on_recept: UnboundedSender<NetworkMessage>) -> Self {
        let (outbound_send, outbound_recv) = mpsc::unbounded_channel();
        // let (inbound_send, inbound_recv) = mpsc::channel(16);
        Self {
            thread: tokio::spawn(Self::work(address.clone(), on_recept, outbound_recv)),
            address,
            sender: outbound_send,
            // receiver: inbound_recv,
        }
    }
}
