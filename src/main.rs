// In rust we code
// In code we trust
// AgarthaSoftware - 2024

mod config;

mod commands;

use clap::Parser;

#[cfg(target_os = "windows")]
pub const INSTANCE_PATH: &str = "%APPDATA%/local/wormhole";

#[cfg(target_os = "linux")]
pub const INSTANCE_PATH: &'static str= "/usr/local/share/wormhole/";

#[derive(Parser)] // requires `derive` feature
#[command(name = "wormhole")]
#[command(bin_name = "wormhole")]
enum CargoCli {
    /// make a pod and join a network
    Join(JoinArgs),
    /// make a pod and create a new network
    Create(CreateArgs),
    /// remove a pod from its network
    Remove(RemoveArgs),
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct PodArgs {
    /// Change to DIRECTORY before doing anything
    #[arg(long, short='C')]
    path: Option<std::path::PathBuf>,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct JoinArgs {
    /// network url as <address of node to join from> + ':' + <network name>'
    #[arg()]
    url: String,
    /// additional hosts to try to join from as a backup
    #[arg()]
    additional_hosts: Option<Vec<String>>,
    /// Change to DIRECTORY before doing anything
    #[arg(long, short='C')]
    path: Option<std::path::PathBuf>,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct CreateArgs {
    /// name of the network to create
    #[arg()]
    name: Option<String>,
    /// Change to DIRECTORY before doing anything
    #[arg(long, short='C')]
    path: Option<std::path::PathBuf>,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct RemoveArgs {
    /// name of the network to create
    #[arg(short='x', group="mode")]
    take: bool,
    #[arg(short='c', group="mode")]
    clone: bool,
    #[arg(short='d', group="mode")]
    delete: bool,
    /// Change to DIRECTORY before doing anything
    #[arg(long, short='C')]
    path: Option<std::path::PathBuf>,
}


fn main() -> Result<(), Box<dyn std::error::Error>>{
    match CargoCli::parse() {
        CargoCli::Join(args) => {
            println!("joining {}", args.url);
            println!("({:?})", args.additional_hosts);
            commands::join(&args.path.unwrap_or(".".into()), args.url, args.additional_hosts.unwrap_or(vec!()))?;
        },
        CargoCli::Create(args) => println!("creating network {:?}", args.name),
        CargoCli::Remove(args) => {
            println!("removing pod");
            let mode = match (args.clone, args.delete, args.take) {
                (true, false, false) => commands::Mode::Clone,
                (false, true, false) => commands::Mode::Clean,
                (false, false, true) => commands::Mode::Take,
                (false, false, false) => commands::Mode::Simple,
                _ => unreachable!("multiple exclusive options"),
            };
            commands::remove(&args.path.unwrap_or(".".into()), mode)?;
        },
    }
    Ok(())
}
