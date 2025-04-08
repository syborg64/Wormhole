use custom_error::custom_error;

custom_error! {pub WhError
    InodeNotFound = "Entry not found",
    DeadLock = "A DeadLock occured",
    NetworkDied{called_from: String} = @{format!("{called_from}: Unable to update modification on the network thread")},
    WouldBlock{called_from: String} = @{format!("{called_from}: Unable to lock arbo")}
}

impl WhError {
    //TODO move to own file
    pub fn to_libc(&self) -> i32 {
        match self {
            WhError::InodeNotFound => libc::ENOENT,
            WhError::DeadLock => libc::EDEADLOCK,
            WhError::NetworkDied { called_from: _ } => libc::ENETDOWN,
            WhError::WouldBlock { called_from: _ } => libc::SIG_BLOCK,
        }
    }
}

pub type WhResult<T> = Result<T, WhError>;
