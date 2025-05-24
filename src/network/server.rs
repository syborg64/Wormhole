use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tokio::{net::TcpListener, sync::mpsc::UnboundedReceiver};

use crate::error::{CliError, WhError, WhResult};

use super::message::ToNetworkMessage;

pub type Tx = UnboundedReceiver<ToNetworkMessage>;
pub type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

pub struct Server {
    pub listener: TcpListener,
    pub state: PeerMap,
}

//TODO - ne pas panic mais renvoyer une erreur
impl Server {
    pub async fn setup(addr: &str) -> Result<Server, CliError> {
        Ok(Server {
            listener: TcpListener::bind(addr).await.map_err(|_| CliError::Server { addr: addr.to_owned() })?,
            state: PeerMap::new(Mutex::new(HashMap::new())),
        })
    }
}
