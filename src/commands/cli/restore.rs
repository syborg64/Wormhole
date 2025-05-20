use std::env;

use tokio::runtime::Runtime;

use crate::{commands::cli_commands::{Cli, RestoreConf}, error::CliError};

use super::cli_messager;

pub fn restore(ip: &str, mut args: RestoreConf) -> Result<(), Box<dyn std::error::Error>> {
  let files_name = vec![".local_config.toml", ".global_config.toml"];
    
    for file in args.files.clone() {
      if !files_name.contains(&file.as_str()) {
        return Err(Box::new(CliError::InvalidArgument { arg: file }));
      }
    }
    if args.name == "." {
      let path = env::current_dir()?;
      args.path.inner = path.display().to_string();
      args.path.clone().set_absolute();
    }
    let rt = Runtime::new().unwrap();
    rt.block_on(cli_messager(
      ip,
      Cli::Restore(RestoreConf { name: args.name, path: args.path, files: args.files }),
    ))
  }