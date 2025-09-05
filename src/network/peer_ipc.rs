use futures::future::Either;
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

use crate::{config::{GlobalConfig, LocalConfig}, network::{forward::{forward_peer_to_receiver, forward_sender_to_peer}, handshake::{self, Accept, Handshake, HandshakeError}}, pods::{arbo::Arbo, network::network_interface::NetworkInterface}};

use super::message::{Address, FromNetworkMessage, MessageAndStatus};

#[derive(Debug)]
pub struct PeerIPC {
    pub url: Option<String>,
    pub hostname: String,
    pub thread: tokio::task::JoinHandle<()>,
    pub sender: mpsc::UnboundedSender<MessageAndStatus>,
    // pub receiver: mpsc::Receiver<NetworkMessage>, // receive a message from the peer
}

impl PeerIPC {
    async fn work(
        peer_write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
        peer_read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        receiver_in: mpsc::UnboundedSender<FromNetworkMessage>,
        mut sender_out: mpsc::UnboundedReceiver<MessageAndStatus>,
        peer: String,
    ) {
        tokio::join!(
            forward_peer_to_receiver(peer_read, receiver_in, peer.clone()),
            forward_sender_to_peer(peer_write, &mut sender_out, peer)
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

    pub async fn accept(
        network_interface: &NetworkInterface,
        stream: WebSocketStream<TcpStream>,
        // address: Address,
        // hostname: String,
        receiver_in: UnboundedSender<FromNetworkMessage>,
        // write: SplitSink<WebSocketStream<TcpStream>, Message>,
        // read: SplitStream<WebSocketStream<TcpStream>>,
    ) -> Result<Self, HandshakeError> {
        let (sender_in, sender_out) = mpsc::unbounded_channel();

        let (mut sink, mut stream) = stream.split();

        let (hostname, url) = match handshake::accept(&mut stream, &mut sink, network_interface).await? {
            Either::Left(connect) => (connect.hostname, connect.url),
            Either::Right(wave) => (wave.hostname, wave.url),
        };

        Ok(Self {
            thread: tokio::spawn(Self::work_from_incomming(
                sink,
                stream,
                receiver_in,
                sender_out,
                hostname.clone(),
            )),
            url,
            sender: sender_in,
            hostname,
        })
    }

    pub async fn connect(
        url: String,
        config: &LocalConfig,
        receiver_in: UnboundedSender<FromNetworkMessage>,
    ) -> Result<(Self, Accept), HandshakeError> {
        let _ = config;
        let (sender_in, sender_out) = mpsc::unbounded_channel();

        log::trace!("connecting to ws://{url}");
        let (accept, thread) = match tokio_tungstenite::connect_async_with_config(
            "ws://".to_string() + &url,
            Some(
                WebSocketConfig::default()
                    .max_message_size(None)
                    .max_frame_size(None),
            ),
            false,
        )
        .await
        {
            Ok((stream, _)) => {
                let (mut sink, mut stream) = stream.split();
                let accept = handshake::connect(&mut stream, &mut sink, config.general.hostname.clone()).await?;
                (accept, tokio::spawn(Self::work(sink, stream, receiver_in, sender_out, url.clone())))
            }
            Err(e) => {
                log::warn!("failed to connect to {}. Error: {}", url, e);
                return Err(e.into());
            }
        };
        Ok((Self {
            thread,
            url: Some(url),
            hostname: accept.hostname.clone(),
            sender: sender_in,
        }, accept))
    }

    // pub async fn link()

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
        log::debug!("Dropping PeerIPC {}", self.hostname);
        self.thread.abort();
    }
}
