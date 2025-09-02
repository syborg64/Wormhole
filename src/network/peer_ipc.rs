use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedSender},
};
use tokio_tungstenite::tungstenite::{protocol::WebSocketConfig, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::network::forward::{forward_peer_to_receiver, forward_sender_to_peer};

use super::message::{Address, FromNetworkMessage, MessageAndStatus};

#[derive(Debug)]
pub struct PeerIPC {
    pub address: Address,
    pub thread: tokio::task::JoinHandle<()>,
    pub sender: mpsc::UnboundedSender<MessageAndStatus>, // send a message to the peer
                                                         // pub receiver: mpsc::Receiver<NetworkMessage>, // receive a message from the peer
}

impl PeerIPC {
    async fn work(
        peer_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        receiver_in: mpsc::UnboundedSender<FromNetworkMessage>,
        mut sender_out: mpsc::UnboundedReceiver<MessageAndStatus>,
        peer: String,
    ) {
        let (write, read) = peer_stream.split();
        tokio::join!(
            forward_peer_to_receiver(read, receiver_in, peer.clone()),
            forward_sender_to_peer(write, &mut sender_out, peer)
        );
    }

    async fn work_from_incomming(
        peer_write: SplitSink<WebSocketStream<TcpStream>, Message>,
        peer_read: SplitStream<WebSocketStream<TcpStream>>,
        receiver_in: mpsc::UnboundedSender<FromNetworkMessage>,
        mut sender_out: mpsc::UnboundedReceiver<MessageAndStatus>,
        peer: Address,
    ) {
        tokio::join!(
            forward_peer_to_receiver(peer_read, receiver_in, peer.clone()),
            forward_sender_to_peer(peer_write, &mut sender_out, peer)
        );
    }

    pub fn connect_from_incomming(
        address: Address,
        receiver_in: UnboundedSender<FromNetworkMessage>,
        write: SplitSink<WebSocketStream<TcpStream>, Message>,
        read: SplitStream<WebSocketStream<TcpStream>>,
    ) -> Self {
        let (sender_in, sender_out) = mpsc::unbounded_channel();
        Self {
            thread: tokio::spawn(Self::work_from_incomming(
                write,
                read,
                receiver_in,
                sender_out,
                address.clone(),
            )),
            address,
            sender: sender_in,
        }
    }

    pub async fn connect(
        address: Address,
        receiver_in: UnboundedSender<FromNetworkMessage>,
    ) -> Option<Self> {
        let (sender_in, sender_out) = mpsc::unbounded_channel();

        let thread = match tokio_tungstenite::connect_async("ws://".to_string() + &address).await {
            Ok((stream, _)) => {
                tokio::spawn(Self::work(stream, receiver_in, sender_out, address.clone()))
            }
            Err(e) => {
                log::warn!("failed to connect to {}. Error: {}", address, e);
                return None;
            }
        };
        Some(Self {
            thread,
            address,
            sender: sender_in,
        })
    }

    // start connexions to peers
    pub async fn peer_startup(
        peers_ip_list: Vec<Address>,
        receiver_in: UnboundedSender<FromNetworkMessage>,
    ) -> Vec<PeerIPC> {
        futures_util::future::join_all(
            peers_ip_list
                .into_iter()
                .map(|ip| PeerIPC::connect(ip, receiver_in.clone())), // .filter(|peer| !peer.thread.is_finished())
        )
        .await
        .into_iter()
        .flatten()
        .collect()
    }
}

impl Drop for PeerIPC {
    fn drop(&mut self) {
        log::debug!("Dropping PeerIPC {}", self.address);
        self.thread.abort();
    }
}
