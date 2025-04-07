use crate::pods::arbo::{Arbo, InodeId};
use crate::pods::interface::fs_interface::FsInterface;
use custom_error::custom_error;

custom_error! {pub WHError
    InodeNotFound = "Entry not found",
    DeadLock = "A DeadLock occured",
    NetworkDied{called_from: String} = @{format!("{called_from}: Unable to update modification on the network thread")},
    WouldBlock{called_from: String} = @{format!("{called_from}: Unable to lock arbo")}
}

impl WHError {
    pub fn to_libc(&self) -> i32 {
        match self {
            WHError::InodeNotFound => libc::ENOENT,
            WHError::DeadLock => libc::EDEADLOCK,
            WHError::NetworkDied { called_from: _ } => libc::ENETDOWN,
            WHError::WouldBlock { called_from: _ } => libc::SIG_BLOCK,
        }
    }
}

custom_error! {pub GetXAttrError
    WHerror{source: WHError} = "{source}",
    KeyNotFound = "Key not found"
}

impl FsInterface {
    pub fn get_inode_xattr(&self, ino: InodeId, key: &String) -> Result<Vec<u8>, GetXAttrError> {
        let arbo = Arbo::n_read_lock(&self.arbo, "fs_interface::get_inode_xattr")?;
        let inode = arbo.n_get_inode(ino)?;

        match inode.xattrs.get(key) {
            Some(data) => Ok(data.clone()),
            None => Err(GetXAttrError::KeyNotFound),
        }
    }

    pub fn xattr_exists(&self, ino: InodeId, key: &String) -> Result<bool, WHError> {
        let arbo = Arbo::n_read_lock(&self.arbo, "fs_interface::get_inode_xattr")?;
        let inode = arbo.n_get_inode(ino)?;

        Ok(inode.xattrs.contains_key(key))
    }

    pub fn set_inode_xattr(&self, ino: InodeId, key: String, data: Vec<u8>) -> Result<(), WHError> {
        self.network_interface.set_inode_xattr(ino, key, data)
    }

    pub fn recept_inode_xattr(
        &self,
        ino: InodeId,
        key: String,
        data: Vec<u8>,
    ) -> std::io::Result<()> {
        self.network_interface.recept_inode_xattr(ino, key, data)
    }
}
