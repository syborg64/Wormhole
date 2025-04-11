use crate::pods::{declarations::Pod, whpath::WhPath};
use clap::Parser;
use serde::{Deserialize, Serialize};

pub enum PodCommand {
    AddPod(String, Pod),
    JoinPod(String, Pod),
    StartPod(StatusPodArgs),
    StopPod(StatusPodArgs),
}

#[derive(Debug, Parser, Serialize, Deserialize)] // requires `derive` feature
#[command(name = "wormhole")]
#[command(bin_name = "wormhole")]
pub enum Cli {
    /// start the service
    Start(StatusPodArgs),
    /// stop the service
    Stop(StatusPodArgs),
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
    /// reload a pod
    Reload(PodArgs),
}

#[derive(Debug, clap::Args, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
pub struct PodArgs {
    pub name: String,
    /// Change to DIRECTORY before doing anything
    #[arg(long, short = 'C')]
    pub path: Option<WhPath>,
}

#[derive(Debug, clap::Args, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
pub struct StatusPodArgs {
    pub name: Option<String>,
    pub path: Option<WhPath>,
}

#[derive(Debug, clap::Args, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
pub struct JoinArgs {
    pub name: String,
    /// network url as <address of node to join from> + ':' + <network name>'
    #[arg()]
    pub url: String,
    /// additional hosts to try to join from as a backup
    #[arg()]
    pub additional_hosts: Option<Vec<String>>,
    /// Change to DIRECTORY before doing anything
    #[arg(long, short = 'C')]
    pub path: Option<WhPath>,
}

#[derive(Debug, clap::Args, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
pub struct TemplateArg {
    /// name of the network to create
    #[arg()]
    pub name: Option<String>,
    /// Change to DIRECTORY before doing anything
    #[arg(long, short = 'C')]
    pub path: Option<WhPath>,
}

#[derive(Debug, clap::Args, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
pub struct RemoveArgs {
    /// name of the network to create
    #[arg(short = 'x', group = "mode")]
    pub take: bool,
    #[arg(short = 'c', group = "mode")]
    pub clone: bool,
    #[arg(short = 'd', group = "mode")]
    pub delete: bool,
    /// Change to DIRECTORY before doing anything
    #[arg(long, short = 'C')]
    pub path: Option<WhPath>,
}
