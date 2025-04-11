use custom_error::custom_error;

use crate::error::WhError;

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
