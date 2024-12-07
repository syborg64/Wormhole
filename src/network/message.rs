use serde::{Deserialize, Serialize};

use crate::{data::metadata::MetaData, providers::Ino};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MessageContent {
    Remove(Ino),
    File(File),
    Meta(MetaData),
    NewFolder(Folder),
    RequestFile(std::path::PathBuf),
    Binary(Vec<u8>),
    Write(Ino, Vec<u8>),
    RequestFs,
    FileStructure(FileStructure),
}

pub type Adress = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FromNetworkMessage {
    pub origin: Adress,
    pub content: MessageContent,
}

/// Networks Messages
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ToNetworkMessage {
    BroadcastMessage(MessageContent),
    SpecificMessage(MessageContent, Vec<Adress>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileStructure {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct File {
    pub path: std::path::PathBuf,
    pub ino: Ino,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Folder {
    pub ino: Ino,
    pub path: std::path::PathBuf,
}
