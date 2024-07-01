// In rust we code
// In code we trust
// AgarthaSoftware - 2024

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
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}


#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct JoinArgs {
    #[arg()]
    url: String,
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct CreateArgs {
    #[arg()]
    name: Option<String>,
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}



fn main() {
    match CargoCli::parse() {
        CargoCli::Join(args) => println!("joining {}", args.url),
        CargoCli::Create(args) => println!("creating network {:?}", args.name),
        CargoCli::Remove(_) => println!("removing pod"),
    }
}
