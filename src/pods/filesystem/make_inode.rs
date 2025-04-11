use custom_error::custom_error;

use crate::{
    error::WhError,
    pods::arbo::{Arbo, FsEntry, Inode},
};

use super::{
    fs_interface::{FsInterface, SimpleFileType},
    remove_inode::RemoveInode,
};

custom_error! {pub MakeInode
    WhError{source: WhError} = "{source}",
    AlreadyExist = "File already existing",
    ParentNotFound = "Parent does not exist",
    ParentNotFolder = "Parent isn't a folder",
    LocalCreationFailed{io: std::io::Error} = "Local creation failed: {io}"
}

impl FsInterface {
    #[must_use]
    /// Create a new empty [Inode], define his informations and register both
    /// in the network and in the local filesystem
    pub fn n_make_inode(
        &self,
        parent_ino: u64,
        name: String,
        kind: SimpleFileType,
    ) -> Result<Inode, MakeInode> {
        let new_entry = match kind {
            SimpleFileType::File => FsEntry::File(vec![self.network_interface.self_addr.clone()]),
            SimpleFileType::Directory => FsEntry::Directory(Vec::new()),
        };

        let new_inode_id = match (name.as_str(), parent_ino) {
            (".global_config.toml", 1) => 2u64,
            (".local_config.toml", 1) => 3u64,
            _ => self.network_interface.n_get_next_inode()?,
        };

        let new_inode = Inode::new(name, parent_ino, new_inode_id, new_entry);

        {
            let arbo = Arbo::n_read_lock(&self.arbo, "make inode")?;
            let parent = arbo.n_get_inode(new_inode.parent)?;
            //check if already exist
            match arbo.n_get_inode_child_by_name(&parent, &new_inode.name) {
                Ok(_) => return Err(MakeInode::AlreadyExist),
                Err(WhError::InodeNotFound) => {}
                Err(err) => return Err(MakeInode::WhError { source: err }),
            }
        }
        //REVIEW: VERY OPINIONATED, register inode as been split to just perform the network call
        //this way we can act if the local creation fails (It might be over the top
        // but prevent network creation if local creation fails, preventing fantom files)
        // 1. add to the arbo
        // 2. create the file
        // IF FAIL
        // 3. Remove from arbo
        // ELSE
        // 3. Register to network

        self.network_interface.n_add_inode(new_inode.clone())?;
        let arbo = Arbo::n_read_lock(&self.arbo, "make inode")?;
        let new_path = arbo.n_get_path_from_inode_id(new_inode_id)?;

        let local_result = match kind {
            SimpleFileType::File => self
                .disk
                .n_new_file(new_path, new_inode.meta.perm)
                .map(|_| ()),
            SimpleFileType::Directory => self.disk.n_new_dir(new_path, new_inode.meta.perm),
        };

        // Remove the inode from the arbo if the local write failed
        if let Err(err) = local_result {
            self.network_interface
                .n_remove_inode_from_arbo(new_inode_id)
                .map_err(|err| match err {
                    RemoveInode::WhError { source } => MakeInode::WhError { source },
                    RemoveInode::NonEmpty => {
                        panic!("Just created folder should never be already filled")
                    }
                })?;
            return Err(MakeInode::LocalCreationFailed { io: err });
        }

        if new_inode_id != 3u64 {
            self.network_interface
                .n_register_new_file(new_inode.clone());
        }
        Ok(new_inode)
    }
}
