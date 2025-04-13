use std::sync::Arc;

use log::error;
use predicates::name;
use tokio::{runtime::Runtime, sync::mpsc};

use crate::{
    commands::{
        cli::cli_messager, cli_commands::PodArgs, default_global_config, default_local_config,
        PodCommand,
    },
    config::{types::Config, GlobalConfig, LocalConfig},
    error::{CliError, CliResult, CliSuccess},
    network::server::Server,
    pods::{declarations::Pod, whpath::WhPath},
};

pub async fn new(tx: mpsc::UnboundedSender<PodCommand>, args: PodArgs) -> CliResult {
    let (global_config, local_config, server, mount_point) = pod_value(&args).await;
    let new_pod = match Pod::new(
        args.name.clone(),
        mount_point,
        1,
        global_config.general.peers,
        server.clone(),
        local_config.general.address,
    )
    .await
    {
        Ok(pod) => pod,
        Err(e) => return Err(CliError::PodCreationFailed { reason: e }),
    };
    match tx.send(PodCommand::JoinPod(new_pod)) {
        Ok(_) => Ok(CliSuccess::PodCreated { pod_id: args.name }),
        Err(e) => Err(CliError::SendCommandFailed {
            reason: e.to_string(),
        }),
    }
}

fn add_hosts(mut global_config: GlobalConfig, additional_hosts: Vec<String>) -> GlobalConfig {
    if additional_hosts.is_empty() {
        return global_config;
    }

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

async fn pod_value(args: &PodArgs) -> (GlobalConfig, LocalConfig, Arc<Server>, WhPath) {
    let global_config: GlobalConfig =
        GlobalConfig::read(".global_config.toml").unwrap_or(default_global_config());
    let local_config: LocalConfig =
        LocalConfig::read(".local_config.toml").unwrap_or(default_local_config(&args.name));
    let server: Arc<Server> = Arc::new(Server::setup(&local_config.general.address).await);

    let global_config = add_hosts(
        global_config,
        args.additional_hosts.clone().unwrap_or(vec![]),
    );

    let mount_point = args.path.clone().unwrap_or_else(|| ".".into());

    (global_config, local_config, server, mount_point)
}
