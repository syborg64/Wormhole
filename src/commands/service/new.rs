use std::sync::Arc;
use std::env;

use log::info;
use tokio::sync::mpsc;

use crate::{
    commands::{
        cli_commands::PodArgs, default_global_config, default_local_config, PodCommand
    },
    config::{types::Config, GlobalConfig, LocalConfig},
    error::{CliError, CliResult, CliSuccess},
    network::server::Server,
    pods::{pod::Pod, whpath::WhPath},
};

pub async fn new(tx: mpsc::UnboundedSender<PodCommand>, args: PodArgs) -> CliResult {
    match pod_value(&args).await {
        Ok((global_config, local_config, server, mount_point)) => {
            println!("local config: {:?}", local_config);
            let new_pod = match Pod::new(
                args.name.clone(),
                global_config,
                mount_point,
                server.clone(),
                local_config.general.address,
            )
            .await
            {
                Ok(pod) => pod,
                Err(e) => return Err(CliError::PodCreationFailed { reason: e }),
            };
            tx.send(PodCommand::NewPod(args.name.clone(), new_pod)).expect("Cli feedback channel is closed");
            Ok(CliSuccess::PodCreated { pod_id: args.name })
        }
        Err(e) => {
            
            Err(e)
        }
    }
}

fn add_hosts(mut global_config: GlobalConfig, url: String, mut additional_hosts: Vec<String>) -> GlobalConfig {
    if url.is_empty() && additional_hosts.is_empty() {
        return global_config;
    }

    additional_hosts.push(url);
    if global_config.general.peers.is_empty() {
        global_config.general.peers = additional_hosts;
    } else {
        for host in additional_hosts {
            if !global_config.general.peers.contains(&host) {
                global_config.general.peers.push(host);
            }
        }
    }
    global_config
}

async fn pod_value(args: &PodArgs) -> Result<(GlobalConfig, LocalConfig, Arc<Server>, WhPath), CliError> {
    let path = if args.path == None {
        let path = env::current_dir()?;
        WhPath::from(&path.display().to_string())
    } else {
        WhPath::from(args.path.clone().unwrap())
    };
    
    let local_path = path.clone().join(".local_config.toml").inner;
    let mut local_config: LocalConfig = LocalConfig::read(&local_path).unwrap_or(default_local_config(&args.name));
    if local_config.general.name != args.name {
        //REVIEW - changer le nom sans prévenir l'utilisateur ou renvoyer une erreur ? Je pense qu'il serait mieux de renvoyer une erreur
        local_config.general.name = args.name.clone();
    }
    if let Err(_) = local_config.write(&local_path) {
        return Err(CliError::InvalidConfig { file: local_path });
    }
    
    let server: Arc<Server> = Arc::new(Server::setup(&local_config.general.address).await);
    
    let global_path = path.clone().join(".global_config.toml").inner;
    let global_config: GlobalConfig = GlobalConfig::read(global_path).unwrap_or(default_global_config());
    let global_config = add_hosts(
        global_config,
        args.url.clone().unwrap_or("".to_string()),
        args.additional_hosts.clone().unwrap_or(vec![]),
    );

    let mount_point = args.path.clone().unwrap_or_else(|| ".".into());
    info!("POD VALUE");
    info!("{:?}", global_config);
    info!("{:?}", local_config);
    Ok((global_config, local_config, server, mount_point))
}
