use crate::error::{WhError, WhResult};
use crate::pods::arbo::{Arbo, InodeId};
use crate::pods::interface::fs_interface::FsInterface;
use custom_error::custom_error;

custom_error! {pub GetXAttrError
    WhError{source: WhError} = "{source}",
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

    pub fn xattr_exists(&self, ino: InodeId, key: &String) -> WhResult<bool> {
        let arbo = Arbo::n_read_lock(&self.arbo, "fs_interface::xattr_exists")?;
        let inode = arbo.n_get_inode(ino)?;

        Ok(inode.xattrs.contains_key(key))
    }

    pub fn set_inode_xattr(&self, ino: InodeId, key: String, data: Vec<u8>) -> WhResult<()> {
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

    pub fn remove_inode_xattr(&self, ino: InodeId, key: String) -> WhResult<()> {
        self.network_interface.remove_inode_xattr(ino, key)
    }

    pub fn recept_remove_inode_xattr(&self, ino: InodeId, key: String) -> std::io::Result<()> {
        self.network_interface.recept_remove_inode_xattr(ino, key)
    }
}
