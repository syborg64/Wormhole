use serde::{Deserialize, Serialize};

use crate::data::metadata::MetaData;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetworkMessage {
    File(File),
    Meta(MetaData),
    RequestFile(std::path::PathBuf),
    Binary(Vec<u8>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct File {
    pub path: std::path::PathBuf,
    pub file: Vec<u8>,
}
