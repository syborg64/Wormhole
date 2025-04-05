// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use tokio::runtime::Runtime;

use crate::{
    commands::{
        self,
        cli::message::cli_messager, cli_commands::{Cli, JoinArgs},
    },
    config::{self, types::Config},
    pods::whpath::WhPath,
};

#[must_use]
pub fn join(
    ip: &str,
    path: &WhPath,
    url: String,
    mut additional_hosts: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let split = url.split(':');
    let slice = &(split.collect::<Vec<_>>())[..];
    if let [address_str, network_name_str] = *slice {
        println!("passed: {:?}", slice);
        let mut peers = vec![address_str.to_owned()];
        peers.append(&mut additional_hosts);
        let network = config::Network::new(peers, network_name_str.to_owned());
        commands::cli::templates(path, network_name_str)?;
        network.write(path.join(".wormhole/network.toml").inner)?;

        let rt = Runtime::new().unwrap();
        rt.block_on(cli_messager(ip, Cli::Join(JoinArgs { url: url.clone(), additional_hosts: None, path: Some(path.clone()) }) ))?;
        return Ok(());
    } else {
        println!("errored: {:?}", slice);
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "url invalid",
        )));
    }
}
