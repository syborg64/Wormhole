// In rust we code
// In code we trust
// AgarthaSoftware - 2024

use crate::commands;
use crate::config::types::Config;
use crate::pods::whpath::WhPath;
use std::error::Error;
use std::fs;

#[derive(PartialEq)]
pub enum Mode {
    /// Simply remove the pod from the network without losing any data from the network
    /// and leaving behind any data that was stored on the pod
    Simple,
    /// remove the pod from the network without losing any data on the network,
    /// and clone all data from the network into the folder where the pod was
    /// making this folder into a real folder
    Clone,
    /// remove the pod from the network and delete any data that was stored in the pod
    Clean,
    /// remove this pod from the network without distributing its data to other nodes
    Take,
}

#[must_use]
pub fn remove(path: &WhPath, mode: Mode) -> Result<(), Box<dyn Error>> {
    if mode != Mode::Take {
        println!("todo!: implement redistribute");
    }
    if mode == Mode::Clone {
        todo!("clone")
    }

    let name = crate::config::Network::read(path.join(".wormhole/network.toml").inner)?.name;

    commands::cli::unregister(&name)?;
    fs::remove_dir_all((&path).join(".wormhole").inner)?;
    if mode == Mode::Clean {
        fs::remove_dir_all(path.inner.clone())?;
    }
    Ok(())
}
