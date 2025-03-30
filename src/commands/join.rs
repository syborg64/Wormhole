// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use tokio::runtime::Runtime;

use crate::{
    commands::{
        self,
        message::{cli_messager, CliMessage},
    },
    config::{self, types::Config},
    pods::whpath::WhPath,
};

#[must_use]
pub fn join(
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
        commands::templates(path, network_name_str)?;
        network.write(path.join(".wormhole/network.toml").inner)?;

        let rt = Runtime::new().unwrap();
        rt.block_on(cli_messager(CliMessage {
            command: "join".to_string(),
        }))?;
        return Ok(());
    } else {
        println!("errored: {:?}", slice);
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "url invalid",
        )));
    }
}
