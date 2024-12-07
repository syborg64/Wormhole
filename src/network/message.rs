use serde::{Deserialize, Serialize};

use crate::{data::metadata::MetaData, providers::Ino};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetworkMessage {
    Remove(Ino),
    File(File),
    Meta(MetaData),
    NewFolder(Folder),
    RequestFile(std::path::PathBuf),
    RequestArborescence,
    Binary(Vec<u8>),
    Write(Ino, Vec<u8>),
}

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
