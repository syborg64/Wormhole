use crate::pods::arbo::{Arbo, InodeId};
use crate::pods::interface::fs_interface::FsInterface;
use custom_error::custom_error;

custom_error! {pub WHError
    InodeNotFound = "Entry not found",
    DeadLock = "A DeadLock occured",
    WouldBlock{called_from: String} = @{format!("{}: unable to read_lock arbo", called_from)}
}

impl WHError {
    pub fn to_libc(&self) -> i32 {
        match self {
            WHError::InodeNotFound => libc::ENOENT,
            WHError::DeadLock => libc::EDEADLOCK,
            WHError::WouldBlock { called_from: _ } => libc::SIG_BLOCK,
        }
    }
}

custom_error! {pub XAttrError
    WHerror{source: WHError} = "{source}",
    KeyNotFound = "Key not found"
}

impl FsInterface {
    pub fn get_inode_x_attribute(&self, ino: InodeId, key: &String) -> Result<Vec<u8>, XAttrError> {
        let arbo = Arbo::n_read_lock(&self.arbo, "fs_interface::get_inode_x_attribute")?;
        let inode = arbo.n_get_inode(ino)?;

        match inode.xattrs.get(key) {
            Some(data) => Ok(data.clone()),
            None => Err(XAttrError::KeyNotFound),
        }
    }

    // pub fn set_inode_x_attribute(
    //     &self,
    //     ino: InodeId,
    //     key: &String,
    //     data: &[u8],
    // ) -> Result<(), XAttrError> {
    // }
}
