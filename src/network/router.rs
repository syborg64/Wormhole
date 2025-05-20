use std::{collections::HashMap, sync::Arc};

use parking_lot::{Mutex, RwLock};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::pods::arbo::LOCK_TIMEOUT;

use super::{
    ip::IpP,
    message::{FromNetworkMessage, ServiceMessage},
    peer_ipc::PeerIPC,
    server::Server,
};

pub struct Router {
    waiting_registeration: Arc<Mutex<Vec<PeerIPC>>>,
    registered_services: Arc<Mutex<HashMap<IpP, PeerIPC>>>,
    to_pods_tx: HashMap<String, UnboundedSender<FromNetworkMessage>>,
}

impl Router {
    pub fn new(self_addr: Address) -> (Self, UnboundedSender<ServiceMessage>) {}
}

pub async fn incoming_services_connections_watchdog(
    server: Arc<Server>,
    to_airport: UnboundedSender<ServiceMessage>,
    waiting_registeration: Arc<Mutex<Vec<PeerIPC>>>,
) {
    while let Ok((stream, addr)) = server.listener.accept().await {
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");

        let (write, read) = futures_util::StreamExt::split(ws_stream);
        let new_peer = PeerIPC::connect_from_incomming(to_airport.clone(), write, read);
        {
            waiting_registeration
                .try_lock_for(LOCK_TIMEOUT)
                .expect("incoming_services_connections_watchdog: can't lock existing peers")
                .push(new_peer);
        }
    }
}

pub async fn services_network_airport(mut network_reception: UnboundedReceiver<ServiceMessage>) {
    loop {
        let message = match network_reception.recv().await {
            Some(message) => message,
            None => continue,
        };

        match message {
            ServiceMessage::Register(as_addr)
        };
    }
}
