use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedSender},
};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

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
        stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        sender: mpsc::UnboundedSender<NetworkMessage>,
        mut receiver: mpsc::UnboundedReceiver<NetworkMessage>,
    ) {
        let (write, read) = stream.split();
        tokio::join!(
            forward_read_to_sender(read, sender),
            forward_receiver_to_write(write, &mut receiver)
        );
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
        let (peer_send, peer_recv) = mpsc::unbounded_channel();

        Self {
            thread: tokio::spawn(Self::work_from_incomming(
                write,
                read,
                on_recept,
                peer_recv,
            )),
            address,
            sender: peer_send,
        }
    }

    pub async fn connect(
        address: String,
        nfa_tx: UnboundedSender<NetworkMessage>,
    ) -> Option<Self> {
        let (peer_send, peer_recv) = mpsc::unbounded_channel();

        let thread = match tokio_tungstenite::connect_async(&address).await {
            Ok((stream, _)) => tokio::spawn(Self::work(stream, nfa_tx, peer_recv)),
            Err(e) => {
                println!("failed to connect to {}. Error: {}", address, e);
                return None;
            }
        };
        Some(Self {
            thread,
            address,
            sender: peer_send,
            // receiver: inbound_recv,
        })
    }
}
