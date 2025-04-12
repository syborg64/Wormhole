// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use std::env;

use tokio::runtime::Runtime;

use crate::{
    commands::{
        self,
        cli::message::cli_messager,
        cli_commands::{Cli, PodArgs},
    },
    config::{self, types::Config},
    pods::whpath::WhPath,
};

use super::init;

#[must_use]
pub fn new(ip: &str, args: PodArgs) -> Result<(), Box<dyn std::error::Error>> {
    let path = if args.path == None {
        let path = env::current_dir()?;
        WhPath::from(&path.display().to_string())
    } else {
        WhPath::from(args.path.unwrap())
    };
    if (args.url == None) && (args.additional_hosts == None) {
        init(ip, args.name, &path)?;
        return Ok(());
    } else {
        let url = args.url.unwrap();
        let mut additional_hosts = args.additional_hosts.unwrap_or(vec![]);
        let split = url.split(':');
        let slice = &(split.collect::<Vec<_>>())[..];
        if let [address_str, network_name_str] = *slice {
            println!("passed: {:?}", slice);
            let mut peers = vec![address_str.to_owned()];
            peers.append(&mut additional_hosts);
            let network = config::Network::new(peers, network_name_str.to_owned());
            commands::cli::templates(&path, network_name_str)?;
            network.write(path.join(".wormhole/network.toml").inner)?;

            let rt = Runtime::new().unwrap();
            rt.block_on(cli_messager(
                ip,
                Cli::New(PodArgs {
                    name: args.name.clone(),
                    path: Some(path.clone()),
                    url: Some(url.clone()),
                    additional_hosts: None,
                }),
            ))?;
            return Ok(());
        } else {
            println!("errored: {:?}", slice);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "url invalid",
            )));
        }
    }
}
