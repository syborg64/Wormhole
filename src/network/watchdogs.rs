use std::sync::{Arc, Mutex};

use futures_util::StreamExt;
use tokio::{
    io::AsyncReadExt,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

use crate::network::{message::ToNetworkMessage, peer_ipc::PeerIPC};
use crate::providers::Provider;

use crate::network::server::Server;

use super::message::{FromNetworkMessage, MessageContent};

pub async fn local_cli_watchdog() {
    let mut stdin = tokio::io::stdin();
    let mut buf = vec![0; 1024];

    loop {
        let read = stdin.read(&mut buf).await;

        // NOTE -  on ctrl-D -> quit
        match read {
            Err(_) | Ok(0) => {
                println!("Quiting!");
                break;
            }
            _ => (),
        };
    }
}

/**DOC
 * reads a message (supposely emitted by a peer) related to files actions
 * and execute instructions on the disk
 *
 * params:
 *  @nfa_rx: reception for file related messages
 *  @provider: fuse instance
*/
pub async fn network_file_actions(
    mut nfa_rx: UnboundedReceiver<FromNetworkMessage>,
    provider: Arc<Mutex<Provider>>,
) {
    loop {
        match nfa_rx.recv().await {
            // Some(MessageContent::Binary(bin)) => {
            Some(FromNetworkMessage {
                origin: _,
                content: MessageContent::Binary(bin),
            }) => {
                println!("peer: {:?}", String::from_utf8(bin).unwrap_or_default());
            }
            Some(FromNetworkMessage {
                origin: _,
                content: MessageContent::NewFolder(folder),
            }) => {
                println!("peer: NEW FOLDER");
                let mut provider = provider.lock().expect("failed to lock mutex");
                provider.new_folder(folder.ino, folder.path);
            }
            Some(FromNetworkMessage {
                origin: _,
                content: MessageContent::File(file),
            }) => {
                println!("peer: NEW FILE");
                let mut provider = provider.lock().expect("failed to lock mutex");
                provider.new_file(file.ino, file.path);
            }
            Some(FromNetworkMessage {
                origin: _,
                content: MessageContent::Remove(ino),
            }) => {
                println!("peer: REMOVE");
                let mut provider = provider.lock().expect("failed to lock mutex");
                provider.recpt_remove(ino);
            }
            Some(FromNetworkMessage {
                origin: _,
                content: MessageContent::Write(ino, data),
            }) => {
                println!("peer: WRITE");
                let mut provider = provider.lock().expect("failed to lock mutex");
                provider.recpt_write(ino, data);
            }
            Some(FromNetworkMessage {
                origin: _,
                content: MessageContent::Meta(_),
            }) => {
                println!("peer: META");
            }
            Some(FromNetworkMessage {
                origin: _,
                content: MessageContent::RequestFile(_),
            }) => {
                println!("peer: REQUEST FILE");
            }
            Some(FromNetworkMessage {
                origin: _,
                content: MessageContent::RequestFs,
            }) => {
                let mut provider = provider.lock().expect("failed to lock mutex");
                //provider.tx.send(ToNetworkMessage::SpecificMessage((), ()));
                println!("Arbo requested");
            }
            Some(FromNetworkMessage {
                origin: _,
                content: MessageContent::FileStructure(_),
            }) => {
                //let mut provider = provider.lock().expect("failed to lock mutex");
                println!("Arbo recieved");
            }
            None => {
                () //REVIEW - Is it ok to loop every time ? the recv should wait or throw None every time ?
            }
        };
    }
}

pub async fn incoming_connections_watchdog(
    server: Server,
    nfa_tx: UnboundedSender<FromNetworkMessage>,
    existing_peers: Arc<Mutex<Vec<PeerIPC>>>,
) {
    while let Ok((stream, _)) = server.listener.accept().await {
        println!("connecting new client");
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");
        let addr = ws_stream.get_ref().peer_addr().unwrap().to_string();
        let (write, read) = ws_stream.split();
        let new_peer = PeerIPC::connect_from_incomming(addr, nfa_tx.clone(), write, read);
        {
            existing_peers.lock().unwrap().push(new_peer);
        }
        println!("new client connected");
        // tokio::join!(
        //     forward_read_to_sender(read, nfa_tx.clone()),
        //     forward_receiver_to_write(write, &mut user_rx)
        // );
    }
}
