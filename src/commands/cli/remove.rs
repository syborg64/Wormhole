// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use tokio::runtime::Runtime;

use crate::commands::cli_commands::{Cli, Mode, RemoveArgs};
use crate::pods::whpath::WhPath;
use std::{env, fs};

use super::cli_messager;

#[must_use]
pub fn remove(ip: &str, mut args: RemoveArgs) -> Result<(), Box<dyn std::error::Error>> {
    let path = if args.name == None && args.path == None {
        let path = env::current_dir()?;
        args.path = Some(WhPath::from(&path.display().to_string()));
        path.display().to_string()
    } else {
        args.path.clone().unwrap_or(WhPath::new()).inner
    };
    if args.mode == Mode::Clean {
        fs::remove_dir_all(path)?;
    }
    let rt = Runtime::new().unwrap();
    rt.block_on(cli_messager(ip, Cli::Remove(args)))
}
