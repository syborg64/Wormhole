use custom_error::custom_error;
use std::{fmt, io};

custom_error! {pub WhError
    InodeNotFound = "Entry not found",
    InodeIsNotADirectory = "Entry is not a directory",
    DeadLock = "A DeadLock occured",
    NetworkDied{called_from: String} = @{format!("{called_from}: Unable to update modification on the network")},
    WouldBlock{called_from: String} = @{format!("{called_from}: Unable to lock arbo")},
}

impl WhError {
    pub fn to_libc(&self) -> i32 {
        match self {
            WhError::InodeNotFound => libc::ENOENT,
            WhError::InodeIsNotADirectory => libc::ENOTDIR,
            WhError::DeadLock => libc::EDEADLOCK,
            WhError::NetworkDied { called_from: _ } => libc::ENETDOWN,
            WhError::WouldBlock { called_from: _ } => libc::EWOULDBLOCK,
        }
    }
}

pub type WhResult<T> = Result<T, WhError>;

custom_error! {pub CliError
    SendCommandFailed{reason: String} = "Failed to send command: {reason}",
    PodCreationFailed{reason: io::Error} = "Pod creation failed: {reason}",
    PodRemovalFailed{reason: String} = "Pod removal failed: {reason}",
    InvalidConfig{file: String} = "Configuration file {file} is missing or invalid",
    InvalidCommand = "Unrecognized command",
    InvalidArgument{arg: String} = "Invalid Argument: {arg} is not recognized",
    SystemError{source: WhError} = "System error: {source}", // Intégrer WhError
    IoError{source: io::Error} = "I/O error: {source}" // Pour les erreurs fs::remove_dir_all, etc.
    
}

#[derive(Debug, Clone)]
pub enum CliSuccess {
    /// Succès avec un simple message
    Message(String),
    /// Succès avec un message et des données supplémentaires
    WithData { message: String, data: String },
    /// Succès spécifique, comme la création d’un objet
    PodCreated { pod_id: String },
}

impl fmt::Display for CliSuccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliSuccess::Message(msg) => write!(f, "{}", msg),
            CliSuccess::WithData { message, data } => write!(f, "{} - Données: {}", message, data),
            CliSuccess::PodCreated { pod_id } => write!(f, "Pod créé avec succès (ID: {})", pod_id),
        }
    }
}

pub type CliResult = Result<CliSuccess, CliError>;
