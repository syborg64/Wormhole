// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use std::{env, fs};

use tokio::runtime::Runtime;

use crate::{
    commands::{
        cli::message::cli_messager,
        cli_commands::{Cli, PodArgs},
    },
    pods::whpath::WhPath,
};

fn check_config_file(
    path: &WhPath,
    files_name: Vec<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    for file_name in files_name {
        println!("file_name: {}", path.clone().push(file_name).inner);
        if fs::metadata(path.clone().push(file_name).inner.clone()).is_err() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("The file config {} does not exist", file_name),
            )));
        }
    }
    Ok(())
}

pub fn new(ip: &str, args: PodArgs) -> Result<(), Box<dyn std::error::Error>> {
    let path = if args.path == ".".into() {
        let path = env::current_dir()?;
        WhPath::from(&path.display().to_string())
    } else {
        WhPath::from(args.path)
    };
    fs::read_dir(&path.inner)?;
    if args.url == None {
        println!("url: {:?}", args.url);
        let files_name = vec![".local_config.toml", ".global_config.toml"];
        check_config_file(&path, files_name)?;
    }
    let rt = Runtime::new().unwrap();
    rt.block_on(cli_messager(
        ip,
        Cli::New(PodArgs {
            name: args.name,
            path: path.clone(),
            url: args.url,
            additional_hosts: args.additional_hosts,
        }),
    ))
}
