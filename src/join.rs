// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use std::error::Error;
use crate::{config::{self, types::Config}, init};

#[must_use]
pub fn join(path: &std::path::PathBuf, url: String) -> Result<(), Box<dyn Error>> {
    let split = url.split(':');
    let slice = &(split.collect::<Vec<_>>())[..];
    if let [address_str, network_name_str] = *slice {
        println!("passed: {:?}", slice);
        init::init(path, network_name_str)?;
        let network = config::Network::new(vec!(address_str.to_owned()), network_name_str.to_owned());
        network.write((&path).join(".wormhole/network.toml"))?;
        return Ok(());
    } else {
        println!("errored: {:?}", slice);
    }
    Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "url invalid")))
}
