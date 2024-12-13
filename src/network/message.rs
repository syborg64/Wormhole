use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    data::metadata::MetaData,
    pods::arbo::Inode,
    providers::{FsIndex, InodeIndex},
};

/// Message Content
/// Represent the content of the intern message but is also the struct sent
/// through the network
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MessageContent {
    Remove(InodeIndex),
    File(Inode, InodeIndex),
    Meta(MetaData),
    NewFolder(Folder),
    RequestFile(std::path::PathBuf),
    Binary(Vec<u8>),
    Write(InodeIndex, Vec<u8>),
    RequestFs,
    FileStructure(FileSystemSerialized),
}

pub type Address = String;

/// Message Coming from Network
/// Messages recived by peers, forwared to [crate::network::watchdogs::network_file_actions]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FromNetworkMessage {
    pub origin: Address,
    pub content: MessageContent,
}

/// Message Going To Network
/// Messages sent from fuser to process communicating to the peers
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ToNetworkMessage {
    BroadcastMessage(MessageContent),
    SpecificMessage(MessageContent, Vec<Address>),
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileSystemSerialized {
    #[serde_as(as = "Vec<(_, _)>")]
    pub fs_index: FsIndex,
    pub next_inode: InodeIndex,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct File {
    pub path: std::path::PathBuf,
    pub ino: InodeIndex,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Folder {
    pub ino: InodeIndex,
    pub path: std::path::PathBuf,
}
