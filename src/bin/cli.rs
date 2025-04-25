// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use clap::Parser;
use std::env;
use wormhole::commands::{
        self,
        cli_commands::Cli,
    };

fn get_args(args: Vec<String>) -> (String, Vec<String>) {
    // Déterminer l'adresse IP et les arguments pour Cli
    let ip: String;
    let cli_args: Vec<String>;

    if let Some(first_arg) = args.get(1) {
        // Vérifier si le premier argument ressemble à une adresse IP (contient ':')
        if first_arg.contains(':') {
            // C'est probablement une IP, la consommer
            ip = first_arg.clone();
            // Les arguments pour Cli commencent après l'IP
            cli_args = args.into_iter().skip(1).collect();
        } else {
            // Pas une IP, utiliser la valeur par défaut
            ip = "127.0.0.1:8081".to_string();
            // Les arguments pour Cli commencent dès le premier argument
            cli_args = args;
        }
    } else {
        // Aucun argument fourni, utiliser l'IP par défaut et cli_args vide
        ip = "127.0.0.1:8081".to_string();
        cli_args = vec![];
    }
    return (ip, cli_args);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Récupérer tous les arguments
    let args: Vec<String> = env::args().collect();
    let (ip, cli_args) = get_args(args);
    let ip = ip.as_str();
    println!("Starting cli on {}", ip);
    println!("cli args: {:?}", cli_args);

    match Cli::parse_from(cli_args) {
        Cli::Start(args) => commands::cli::start(ip, args)?,
        Cli::Stop(args) => commands::cli::stop(ip, args)?,
        Cli::Template(args) => {
            println!(
                "creating network {:?}",
                args.name.clone()
            );
            commands::cli::templates(
                &args.path,
                &args.name,
            )?;
        }
        Cli::New(args) => {
            println!("creating pod");
            commands::cli::new(ip, args)?;
        }
        Cli::Remove(args) => {
            println!("removing pod");
            commands::cli::remove(ip, args)?;
        }
        Cli::Inspect => {
            println!("inspecting pod");
            todo!("inspect");
        }
        Cli::Reload(_args) => {
            println!("reloading pod");
            todo!("reload");
        }
        Cli::Restore(_arg) => {
            println!("retore a specifique file config");
            todo!("restore");
        }
    }
    Ok(())
}
