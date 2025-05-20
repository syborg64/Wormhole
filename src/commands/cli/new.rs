// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use std::{env, fs};

use tokio::runtime::Runtime;

use crate::{
    commands::{
        cli::message::cli_messager,
        cli_commands::{Cli, PodArgs}, default_local_config,
    }, config::{types::Config, LocalConfig}, error::CliError, pods::whpath::WhPath
};

fn mod_file_conf_content(path: WhPath, name: String, ip: &str) -> Result<(), CliError> {
    let local_path = path.clone().join(".local_config.toml").inner;
    let mut local_config: LocalConfig = LocalConfig::read(&local_path).unwrap_or(default_local_config(&name));
    if local_config.general.name != name {
        //REVIEW - changer le nom sans prévenir l'utilisateur ou renvoyer une erreur ? Je pense qu'il serait mieux de renvoyer une erreur
        local_config.general.name = name.clone();
    }
    if ip != "127.0.0.1:8080" {
        local_config.general.address = ip.to_owned();
    }
    if let Err(_) = local_config.write(&local_path) {
        return Err(CliError::InvalidConfig { file: local_path });
    }
    Ok(())
}

fn is_new_wh_file_config(
    path: &WhPath,
    files_name: Vec<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    for file_name in files_name {
        if fs::metadata(path.clone().push(file_name).inner.clone()).is_err() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("The file config {} does not exist", file_name),
            )));
        }
    }
    Ok(())
}


//FIXME - Error id name of the pod not check (can be already exist)
pub fn new(ip: &str, mut args: PodArgs) -> Result<(), Box<dyn std::error::Error>> {
    let p = env::current_dir()?;
    let path = WhPath::from(&p.display().to_string());
    args.path = if args.path.inner != "." {
        path.join(&args.path)
    } else {
        path
    };
    log::info!("PATH: {}", args.path);
    fs::read_dir(&args.path.inner)?;
    if args.url == None {
        println!("url: {:?}", args.url);
        let files_name = vec![".local_config.toml", ".global_config.toml"];
        is_new_wh_file_config(&args.path, files_name)?;
    }
    mod_file_conf_content(args.path.clone(), args.name.clone(), &args.ip)?;
    let rt = Runtime::new().unwrap();
    rt.block_on(cli_messager(
        ip,
        Cli::New(PodArgs {
            name: args.name,
            path: args.path.clone(),
            url: args.url,
            ip: args.ip,
            additional_hosts: args.additional_hosts,
        }),
    ))
}
