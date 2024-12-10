use std::sync::{Arc, Mutex};

use futures_util::future::join_all;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::network::peer_ipc::PeerIPC;

use crate::network::message::{MessageContent, ToNetworkMessage};

use super::message::FromNetworkMessage;

// receive a message on user_rx and broadcast it to all peers
pub async fn contact_peers(
    peers_list: Arc<Mutex<Vec<PeerIPC>>>,
    mut rx: UnboundedReceiver<ToNetworkMessage>,
) {
    // on message reception, broadcast it to all peers senders
    while let Some(message) = rx.recv().await {
        // match message {
        //     NetworkMessage::BroadcastMessage(message) => todo!(),
        //     NetworkMessage::SpecificMessage(message, vec) => todo!(),
        // }
        //generating peers senders
        // REVIEW - should avoid locking peers in future versions, as it more or less locks the entire program
        let peer_tx: Vec<(UnboundedSender<MessageContent>, String)> = peers_list
            .lock()
            .unwrap()
            .iter()
            .map(|peer| (peer.sender.clone(), peer.address.clone()))
            .collect();

        println!("broadcasting message to peers:\n{:?}", message);
        match message {
            ToNetworkMessage::BroadcastMessage(message_content) => {
                peer_tx.iter().for_each(|(channel, address)| {
                    println!("peer: {}", address);
                    channel
                        .send(message_content.clone())
                        .expect(&format!("failed to send message to peer {}", address))
                });
            }
            ToNetworkMessage::SpecificMessage(message_content, origins) => {
                peer_tx
                    .iter()
                    .filter(|&(_, address)| origins.contains(address))
                    .for_each(|(channel, address)| {
                        println!("peer: {}", address);
                        channel
                            .send(message_content.clone())
                            .expect(&format!("failed to send message to peer {}", address))
                    });
            }
        };
    }
}

// start connexions to peers
pub async fn peer_startup(
    peers_ip_list: Vec<String>,
    nfa_tx: UnboundedSender<FromNetworkMessage>,
) -> Vec<PeerIPC> {
    join_all(
        peers_ip_list
            .into_iter()
            .map(|ip| PeerIPC::connect(ip, nfa_tx.clone())), // .filter(|peer| !peer.thread.is_finished())
    )
    .await
    .into_iter()
    .flatten()
    .collect()
}
