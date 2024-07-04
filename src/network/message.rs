use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetworkMessage {
    Change(Change),
    Binary(Vec<u8>),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Change {
    pub path: std::path::PathBuf,
    pub file: Vec<u8>,
}
