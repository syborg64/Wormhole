// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use clap::Parser;
use wormhole::{commands, pods::whpath::WhPath};

#[derive(Parser)] // requires `derive` feature
#[command(name = "wormhole")]
#[command(bin_name = "wormhole")]
enum Cli {
    /// start the service
    Start,
    /// stop the service
    Stop,
    /// create a new network (template)
    Template(TemplateArg),
    /// make a pod and create a new network
    Init(PodArgs),
    /// make a pod and join a network
    Join(JoinArgs),
    /// inspect a pod with its configuration, connections, etc
    Inspect,
    /// remove a pod from its network
    Remove(RemoveArgs),
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct PodArgs {
    /// Change to DIRECTORY before doing anything
    #[arg(long, short = 'C')]
    path: Option<WhPath>,
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
    #[arg(long, short = 'C')]
    path: Option<WhPath>,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct TemplateArg {
    /// name of the network to create
    #[arg()]
    name: Option<String>,
    /// Change to DIRECTORY before doing anything
    #[arg(long, short = 'C')]
    path: Option<WhPath>,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct RemoveArgs {
    /// name of the network to create
    #[arg(short = 'x', group = "mode")]
    take: bool,
    #[arg(short = 'c', group = "mode")]
    clone: bool,
    #[arg(short = 'd', group = "mode")]
    delete: bool,
    /// Change to DIRECTORY before doing anything
    #[arg(long, short = 'C')]
    path: Option<WhPath>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse() {
        Cli::Start => {
            println!("starting service");
            todo!("start");
        }
        Cli::Stop => {
            println!("stoping service");
            todo!("stop");
        }
        Cli::Template(args) => {
            println!(
                "creating network {:?}",
                args.name.clone().unwrap_or("default".into())
            );
            commands::templates(
                &args.path.unwrap_or(".".into()),
                &args.name.unwrap_or("default".into()),
            )?;
        }
        Cli::Init(args) => {
            println!("init service");
            commands::init(&WhPath::from(args.path.unwrap_or(".".into())))?;
            // todo!("init");
        }
        Cli::Join(args) => {
            println!("joining {}", args.url);
            println!("(additional hosts: {:?})", args.additional_hosts);
            commands::join(
                &args.path.unwrap_or(".".into()),
                args.url,
                args.additional_hosts.unwrap_or(vec![]),
            )?;
        }
        Cli::Remove(args) => {
            println!("removing pod");
            let mode = match (args.clone, args.delete, args.take) {
                (true, false, false) => commands::Mode::Clone,
                (false, true, false) => commands::Mode::Clean,
                (false, false, true) => commands::Mode::Take,
                (false, false, false) => commands::Mode::Simple,
                _ => unreachable!("multiple exclusive options"),
            };
            commands::remove(&WhPath::from(args.path.unwrap_or(".".into())), mode)?;
        }
        Cli::Inspect => {
            println!("inspecting pod");
            todo!("inspect");
        }
    }
    Ok(())
}
