use nt_time::FileTime;
use windows::Win32::{
    Foundation::{
        NTSTATUS, STATUS_DEVICE_NOT_READY, STATUS_NOT_A_DIRECTORY, STATUS_OBJECT_NAME_INVALID,
        STATUS_OBJECT_NAME_NOT_FOUND, STATUS_PENDING, STATUS_POSSIBLE_DEADLOCK,
    },
    Storage::FileSystem::{FILE_ATTRIBUTE_ARCHIVE, FILE_ATTRIBUTE_DIRECTORY},
};
use winfsp::{filesystem::FileInfo, FspError};

use crate::{
    error::WhError,
    pods::{arbo::Metadata, filesystem::fs_interface::SimpleFileType, whpath::WhPath},
};

impl TryInto<WhPath> for &winfsp::U16CStr {
    type Error = NTSTATUS;

    fn try_into(self) -> Result<WhPath, Self::Error> {
        match self.to_string() {
            Err(_) => Err(STATUS_OBJECT_NAME_INVALID),
            Ok(string) => Ok(WhPath::from(&string.replace("\\", "/"))),
        }
    }
}

impl WhPath {
    pub fn to_winfsp(&self) -> String {
        self.inner.replace("/", "\\")
    }
}

impl Into<FileInfo> for Metadata {
    fn into(self) -> FileInfo {
        (&self).into()
    }
}

impl Into<FileInfo> for &Metadata {
    fn into(self) -> FileInfo {
        let attributes = match self.kind {
            SimpleFileType::File => FILE_ATTRIBUTE_ARCHIVE,
            SimpleFileType::Directory => FILE_ATTRIBUTE_DIRECTORY,
        };
        let now = FileTime::now();
        FileInfo {
            file_attributes: attributes.0,
            reparse_tag: 0,
            allocation_size: self.size as u64,
            file_size: self.size as u64,
            creation_time: FileTime::try_from(self.crtime).unwrap_or(now).to_raw(),
            last_access_time: FileTime::try_from(self.atime).unwrap_or(now).to_raw(),
            last_write_time: FileTime::try_from(self.mtime).unwrap_or(now).to_raw(),
            change_time: FileTime::try_from(self.ctime).unwrap_or(now).to_raw(),
            index_number: self.ino,
            hard_links: 0,
            ea_size: 0,
        }
    }
}

impl Into<FspError> for &WhError {
    fn into(self) -> FspError {
        Into::<NTSTATUS>::into(self).into()
    }
}

impl Into<FspError> for WhError {
    fn into(self) -> FspError {
        (&self).into()
    }
}

impl Into<NTSTATUS> for &WhError {
    fn into(self) -> NTSTATUS {
        match self {
            WhError::InodeNotFound => STATUS_OBJECT_NAME_NOT_FOUND,
            WhError::InodeIsNotADirectory => STATUS_NOT_A_DIRECTORY,
            WhError::DeadLock => STATUS_POSSIBLE_DEADLOCK,
            WhError::NetworkDied { called_from: _ } => STATUS_DEVICE_NOT_READY,
            WhError::WouldBlock { called_from: _ } => STATUS_PENDING,
        }
    }
}

impl Into<NTSTATUS> for WhError {
    fn into(self) -> NTSTATUS {
        (&self).into()
    }
}
