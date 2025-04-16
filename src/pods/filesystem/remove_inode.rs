use custom_error::custom_error;

use crate::{
    error::WhError,
    pods::arbo::{Arbo, FsEntry, InodeId},
};

use super::fs_interface::FsInterface;

custom_error! {
    /// Error describing the removal of a [Inode] from the [Arbo]
    pub RemoveInode
    WhError{source: WhError} = "{source}",
    NonEmpty = "Can't remove non-empty dir",
}

custom_error! {
    /// Error describing the removal of a [Inode] from the [Arbo] and the local file or folder
    pub RemoveFile
    WhError{source: WhError} = "{source}",
    NonEmpty = "Can't remove non-empty dir",
    LocalDeletionFailed{io: std::io::Error} = "Local Deletion failed: {io}"
}

impl From<RemoveInode> for RemoveFile {
    fn from(value: RemoveInode) -> Self {
        match value {
            RemoveInode::WhError { source } => RemoveFile::WhError { source },
            RemoveInode::NonEmpty => RemoveFile::NonEmpty,
        }
    }
}

impl FsInterface {
    pub fn n_fuse_remove_inode(
        &self,
        parent: InodeId,
        name: &std::ffi::OsStr,
    ) -> Result<(), RemoveFile> {
        let arbo = Arbo::n_read_lock(&self.arbo, "fs_interface::fuse_remove_inode")?;
        let parent = arbo.n_get_inode(parent)?;
        let target = arbo
            .n_get_inode_child_by_name(parent, &name.to_string_lossy().to_string())?
            .id;
        drop(arbo);

        self.n_remove_inode(target)
    }

    pub fn n_remove_inode(&self, id: InodeId) -> Result<(), RemoveFile> {
        let arbo = Arbo::n_read_lock(&self.arbo, "fs_interface::remove_inode")?;
        let to_remove_path = arbo.n_get_path_from_inode_id(id)?;
        let entry = arbo.n_get_inode(id)?.entry.clone();
        drop(arbo);

        match entry {
            FsEntry::File(hosts) if hosts.contains(&self.network_interface.self_addr) => self
                .disk
                .remove_file(to_remove_path)
                .map_err(|io| RemoveFile::LocalDeletionFailed { io })?,
            FsEntry::File(_) => { /* Nothing to do */ }
            FsEntry::Directory(children) if children.is_empty() => self
                .disk
                .remove_dir(to_remove_path)
                .map_err(|io| RemoveFile::LocalDeletionFailed { io })?,
            FsEntry::Directory(_) => return Err(RemoveFile::NonEmpty),
        };

        if id != 3u64 {
            self.network_interface.n_unregister_file(id)?;
        }
        Ok(())
    }
}
