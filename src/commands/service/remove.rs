use std::collections::HashMap;

use tokio::sync::mpsc;

use crate::{
    commands::{cli_commands::{Mode, RemoveArgs}, PodCommand},
    error::{CliError, CliResult, CliSuccess}, pods::pod::Pod,
};

pub async fn remove(args: RemoveArgs, mut pod: Pod) -> CliResult {
        match args.mode {
            Mode::Simple => {
                //TODO - stop the pod
                //TODO - delete the pod
                todo!("simple");
            }
            Mode::Clone => {
                //TODO - clone all data into a folder
                //TODO - stop the pod
                //TODO - delete the pod
                todo!("clone");
            }
            Mode::Clean => {
                //TODO - stop the pod
                //TODO - delete all data
                //TODO - delete the pod
                todo!("clean");
            }
            Mode::Take => {
                //TODO - stop the pod without distributing its data
                //TODO - delete the pod
                todo!("take");
            }
        };
        Ok(CliSuccess::WithData { message: String::from("Pod removed with success: "), data: pod.get_name().to_string() })
}
