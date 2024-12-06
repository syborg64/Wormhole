// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use std::sync::{Arc, Mutex};

pub use crate::network::message::NetworkMessage;

use super::peer_ipc::PeerIPC;

pub fn request_arbo(peers: Arc<Mutex<Vec<PeerIPC>>>) {
    let peer_array = peers.lock().expect("Mutex Poisned");
    if let Some(peer) = peer_array.last() {
        peer.sender
            .send(NetworkMessage::RequestArborescence)
            .expect(&format!("Failed to send message to peer {}", peer.address));
    }
}
