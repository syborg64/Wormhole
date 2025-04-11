// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use clap::Parser;
use std::{env, path::PathBuf, sync::Arc};
use wormhole::{
    commands::{
        self,
        cli_commands::{self, Cli},
    },
    pods::whpath::WhPath,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let ip: String = env::args()
        .nth(2)
        .unwrap_or("127.0.0.1:8081".to_string())
        .into();
    println!("Starting cli on {}", ip);
    match Cli::parse() {
        Cli::Start(args) => commands::cli::start(ip.as_str(), args)?,
        Cli::Stop(args) => commands::cli::stop(ip.as_str(), args)?,
        Cli::Template(args) => {
            println!(
                "creating network {:?}",
                args.name.clone().unwrap_or("default".into())
            );
            commands::cli::templates(
                &args.path.unwrap_or(".".into()),
                &args.name.unwrap_or("default".into()),
            )?;
        }
        Cli::Init(args) => {
            println!("init service");
            commands::cli::init(
                ip.as_str(),
                args.name,
                &WhPath::from(args.path.unwrap_or(".".into())),
            )?;
        }
        Cli::Join(args) => {
            println!("joining {}", args.url);
            println!("(additional hosts: {:?})", args.additional_hosts);
            commands::cli::join(
                ip.as_str(),
                args.name,
                &args.path.unwrap_or(".".into()),
                args.url,
                args.additional_hosts.unwrap_or(vec![]),
            )?;
        }
        Cli::Remove(args) => {
            println!("removing pod");
            let mode = match (args.clone, args.delete, args.take) {
                (true, false, false) => commands::cli::Mode::Clone,
                (false, true, false) => commands::cli::Mode::Clean,
                (false, false, true) => commands::cli::Mode::Take,
                (false, false, false) => commands::cli::Mode::Simple,
                _ => unreachable!("multiple exclusive options"),
            };
            commands::cli::remove(&WhPath::from(args.path.unwrap_or(".".into())), mode)?;
        }
        Cli::Inspect => {
            println!("inspecting pod");
            todo!("inspect");
        }
        Cli::Reload(_args) => {
            println!("reloading pod");
            todo!("reload");
        }
    }
    Ok(())
}
