enum WHError {
    InodeNotFound,
    DeadLock,
}

enum XAttrError {
    WHerror(WHError),
    KeyNotFound,
}

impl FsInterface {
    pub fn get_inode_x_attribute(&self, ino: InodeId, key: &String) -> Result<Vec<u8>, XAttrError> {
        let arbo = Arbo::read_lock(&self.arbo, "fs_interface::get_inode_x_attribute")?;
        let inode = arbo.get_inode(ino)?;

        match inode.xattrs.get(key) {
            Some(data) => Ok(data.clone()),
            None => Err(XAttrError::KeyNotFound)
        }
    }

    pub fn get_inode_x_attribute(&self, ino: InodeId, key: &String) -> Result<Option<Vec<u8>>, XAttrError> {
        let arbo = Arbo::read_lock(&self.arbo, "fs_interface::get_inode_x_attribute")?;
        let inode = arbo.get_inode(ino)?;

        return Ok(inode.xattrs.get(key).map(|value| value.clone()));
    }
}