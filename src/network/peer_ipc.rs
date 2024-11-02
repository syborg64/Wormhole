use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedSender},
};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

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
        } else {
            println!("failed to connect to {}", address);
        }
    }

    async fn work_from_incomming(
        write: SplitSink<WebSocketStream<TcpStream>, Message>,
        read: SplitStream<WebSocketStream<TcpStream>>,
        sender: mpsc::UnboundedSender<NetworkMessage>,
        mut receiver: mpsc::UnboundedReceiver<NetworkMessage>,
    ) {
        tokio::join!(
            forward_read_to_sender(read, sender),
            forward_receiver_to_write(write, &mut receiver)
        );
    }

    pub fn connect_from_incomming(
        address: String,
        on_recept: UnboundedSender<NetworkMessage>,
        write: SplitSink<WebSocketStream<TcpStream>, Message>,
        read: SplitStream<WebSocketStream<TcpStream>>,
    ) -> Self {
        let (outbound_send, outbound_recv) = mpsc::unbounded_channel();

        Self {
            thread: tokio::spawn(Self::work_from_incomming(
                write,
                read,
                on_recept,
                outbound_recv,
            )),
            address,
            sender: outbound_send,
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
