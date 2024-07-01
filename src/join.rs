// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use std::{error::Error, str::FromStr};
use std::fs;
use serde::{Deserialize, Serialize};

use crate::init;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Peers {
    pub peers: Vec<Peer>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Peer {
    pub address: String,
    pub network_name: String,
}

#[must_use]
pub fn join(path: &std::path::PathBuf, url: String) -> Result<(), Box<dyn Error>> {
    let split = url.split(':');
    let slice = &(split.collect::<Vec<_>>())[..];
    if let [address_str, network_name_str] = *slice {
        println!("passed: {:?}", slice);
        init::init(path, network_name_str)?;
        let peer = Peer { address: address_str.to_owned(), network_name: network_name_str.to_owned()};
        let peers = Peers { peers: vec!(peer.clone(), peer.clone()) };
        let serialized = toml::to_string(&peers)?;
        fs::write((&path).join(".wormhole/peers.toml"), serialized)?;
        return Ok(());
    } else {
        println!("errored: {:?}", slice);
    }
    Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "url invalid")))
}
