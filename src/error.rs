use custom_error::custom_error;

custom_error! {pub WHError
    InodeNotFound = "Entry not found",
    DeadLock = "A DeadLock occured",
    NetworkDied{called_from: String} = @{format!("{called_from}: Unable to update modification on the network thread")},
    WouldBlock{called_from: String} = @{format!("{called_from}: Unable to lock arbo")}
}

impl WHError {
    //TODO move to own file
    pub fn to_libc(&self) -> i32 {
        match self {
            WHError::InodeNotFound => libc::ENOENT,
            WHError::DeadLock => libc::EDEADLOCK,
            WHError::NetworkDied { called_from: _ } => libc::ENETDOWN,
            WHError::WouldBlock { called_from: _ } => libc::SIG_BLOCK,
        }
    }
}
