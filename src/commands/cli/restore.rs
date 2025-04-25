use tokio::runtime::Runtime;

use crate::{commands::cli_commands::{Cli, RestoreConf}, error::CliError};

use super::cli_messager;

pub fn restore(ip: &str, args: RestoreConf) -> Result<(), Box<dyn std::error::Error>> {
  let files_name = vec![".local_config.toml", ".global_config.toml"];
    
    for name in args.names.clone() {
      if !files_name.contains(&name.as_str()) {
        return Err(Box::new(CliError::InvalidArgument { arg: name }));
      }
    }
    let rt = Runtime::new().unwrap();
    rt.block_on(cli_messager(
      ip,
      Cli::Restore(RestoreConf { names: args.names }),
    ))
  }