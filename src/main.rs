// In rust we code
// In code we trust
// AgarthaSoftware - 2024

mod config;

mod init;
mod join;

use clap::Parser;

#[derive(Parser)] // requires `derive` feature
#[command(name = "wormhole")]
#[command(bin_name = "wormhole")]
enum CargoCli {
    /// make a pod and join a network
    Join(JoinArgs),
    /// make a pod and create a new network
    Create(CreateArgs),
    /// remove a pod from its network
    Remove(PodArgs),
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

fn main() -> Result<(), Box<dyn std::error::Error>>{
    match CargoCli::parse() {
        CargoCli::Join(args) => {
            println!("joining {}", args.url);
            println!("({:?})", args.additional_hosts);
            join::join(&args.path.unwrap_or(".".into()), args.url, args.additional_hosts.unwrap_or(vec!()))?;
        },
        CargoCli::Create(args) => println!("creating network {:?}", args.name),
        CargoCli::Remove(_) => println!("removing pod"),
    }
    Ok(())
}
