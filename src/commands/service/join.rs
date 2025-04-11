use std::sync::Arc;

use log::error;
use tokio::{runtime::Runtime, sync::mpsc};

use crate::{
    commands::{cli::cli_messager, cli_commands::JoinArgs, PodCommand},
    config::{GlobalConfig, LocalConfig},
    network::server::Server,
    pods::{pod::Pod, whpath::WhPath},
};

use super::read_config;

pub async fn join(
    tx: mpsc::UnboundedSender<PodCommand>,
    join_args: JoinArgs,
) -> Result<String, String> {
    match pod_value(&join_args).await {
        Ok((global_config, local_config, server, mount_point)) => {
            let global_config = if let Some(additional_hosts) = join_args.additional_hosts {
                if global_config.general.peers.is_empty() {
                    return Err("No known peers".to_string());
                }
                add_hosts(global_config, additional_hosts)
            } else {
                global_config
            };
            let new_pod = match Pod::new(
                join_args.name.clone(),
                mount_point,
                global_config.general.peers,
                server.clone(),
                local_config.general.address,
            )
            .await
            {
                Ok(pod) => pod,
                Err(e) => return Err(format!("Pod creation error: {}", e)),
            };
            match tx.send(PodCommand::JoinPod(join_args.name, new_pod)) {
                Ok(_) => Ok("Pod joined successfully".to_string()),
                Err(e) => Err(format!("PodCommand send error: {}", e)),
            }
        }
        Err(e) => {
            error!("Join error: {}", e);
            Err(format!("Join error: {}", e))
        }
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

async fn pod_value(
    join_args: &JoinArgs,
) -> Result<(GlobalConfig, LocalConfig, Arc<Server>, WhPath), String> {
    let (global_config, local_config, server) = match read_config().await {
        Ok(config) => config,
        Err(e) => {
            pull_config(join_args).await.map_err(|e| e.to_string())?;
            read_config().await?
        }
    };

    let global_config = add_hosts(
        global_config,
        join_args.additional_hosts.clone().unwrap_or_default(),
    );

    let mount_point = join_args.path.clone().unwrap_or_else(|| ".".into());

    Ok((global_config, local_config, server, mount_point))
}

async fn pull_config(join_args: &JoinArgs) -> Result<(), String> {
    let rt = Runtime::new().unwrap();
    // rt.block_on(cli_messager(
    //     &join_args.url,
    //     Cli::Init(PodArgs {
    //         name: name,
    //         path: Some(path.clone()),
    //     }),
    // ))?;
    Ok(())
}
