use crate::pods::arbo::{FsEntry, Inode, Metadata};
use crate::pods::fs_interface::{self, FsInterface, SimpleFileType};
use crate::pods::whpath::WhPath;
use fuser::{
    BackgroundSession, FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData,
    ReplyDirectory, ReplyEntry, Request, TimeOrNow,
};
use libc::{EIO, ENOENT};
use log::debug;
use std::ffi::OsStr;
use std::io;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// NOTE - placeholders
const TTL: Duration = Duration::from_secs(1);

const MOUNT_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

pub const TEMPLATE_FILE_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 0,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o777,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};
// ^ placeholders

// const MIRROR_PTH: &str = "./wh_mirror/";

impl Into<FileAttr> for Metadata {
    fn into(self) -> FileAttr {
        FileAttr {
            ino: self.ino,
            size: self.size,
            blocks: self.size,
            atime: self.atime,
            mtime: self.mtime,
            ctime: self.ctime,
            crtime: self.crtime,
            kind: self.kind,
            perm: self.perm,
            nlink: self.nlink,
            uid: self.uid,
            gid: self.gid,
            rdev: self.rdev,
            flags: self.flags,
            blksize: 1,
        }
    }
}

impl Into<Metadata> for FileAttr {
    fn into(self) -> Metadata {
        Metadata {
            ino: self.ino,
            size: self.size,
            blocks: self.blocks,
            atime: self.atime,
            mtime: self.mtime,
            ctime: self.ctime,
            crtime: self.crtime,
            kind: self.kind,
            perm: self.perm,
            nlink: self.nlink,
            uid: self.uid,
            gid: self.gid,
            rdev: self.rdev,
            flags: self.flags,
            blksize: self.blksize,
        }
    }
}

pub struct FuseController {
    pub fs_interface: Arc<FsInterface>,
}

// NOTE for dev purpose while all metadata is not supported
fn inode_to_fuse_fileattr(inode: Inode) -> FileAttr {
    let mut attr: FileAttr = inode.meta.into();
    attr.ino = inode.id;
    attr.kind = match inode.entry {
        FsEntry::Directory(_) => fuser::FileType::Directory,
        FsEntry::File(_) => fuser::FileType::RegularFile,
    };
    attr
}

// REVIEW - should later invest in proper error handling
impl Filesystem for FuseController {
    // READING

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        match self
            .fs_interface
            .get_entry_from_name(parent, name.to_string_lossy().to_string())
        {
            Ok(inode) => {
                reply.entry(&TTL, &inode_to_fuse_fileattr(inode), 0);
            }
            Err(_) => {
                reply.error(ENOENT);
            }
        };
    }

    fn getattr(&mut self, _req: &Request, ino: u64, _: Option<u64>, reply: ReplyAttr) {
        let attrs = self.fs_interface.get_inode_attributes(ino);

        match attrs {
            Ok(attrs) => reply.attr(&TTL, &attrs.into()),
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn setattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        atime: Option<fuser::TimeOrNow>,
        mtime: Option<fuser::TimeOrNow>,
        ctime: Option<std::time::SystemTime>,
        fh: Option<u64>,
        crtime: Option<std::time::SystemTime>,
        chgtime: Option<std::time::SystemTime>,
        bkuptime: Option<std::time::SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        let attrs = match self.fs_interface.get_inode_attributes(ino) {
            Ok(attrs) => Metadata {
                ino: attrs.ino,
                size: if let Some(size) = size {
                    size
                } else {
                    attrs.size
                },
                blocks: attrs.blocks,
                atime: if let Some(atime) = atime {
                    time_or_now_to_system_time(atime)
                } else {
                    attrs.atime
                },
                mtime: if let Some(mtime) = mtime {
                    time_or_now_to_system_time(mtime)
                } else {
                    attrs.mtime
                },
                ctime: if let Some(ctime) = ctime {
                    ctime
                } else {
                    attrs.ctime
                },
                crtime: if let Some(crtime) = crtime {
                    crtime
                } else {
                    attrs.crtime
                },
                kind: attrs.kind,
                perm: attrs.perm,
                nlink: attrs.nlink,
                uid: if let Some(uid) = uid { uid } else { attrs.uid },
                gid: if let Some(gid) = gid { gid } else { attrs.gid },
                rdev: attrs.rdev,
                blksize: attrs.blksize,
                flags: if let Some(flags) = flags {
                    flags
                } else {
                    attrs.flags
                },
            },
            Err(err) => {
                log::error!("fuse_impl::setattr: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO));
                return;
            }
        };

        match self.fs_interface.set_inode_meta(ino, attrs.clone()) {
            Ok(_) => reply.attr(&TTL, &attrs.into()),
            Err(err) => {
                log::error!("fuse_impl::setattr: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        let content = self.fs_interface.read_file(
            ino,
            offset.try_into().expect("fuse_impl::read offset negative"),
            size.try_into().expect("fuse_impl::read size too large"),
        );

        match content {
            Ok(content) => reply.data(&content),
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let entries = match self.fs_interface.read_dir(ino) {
            Ok(entries) => entries,
            Err(e) => {
                log::error!("readdir: ENOENT {e} {ino}");
                reply.error(ENOENT);
                return;
            }
        };

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            if reply.add(
                ino,
                // i + 1 means offset of the next entry
                i as i64 + 1, // NOTE - in case of error, try i + 1
                entry.entry.get_filetype(),
                entry.name,
            ) {
                break;
            }
        }
        reply.ok();
    }

    // ^ READING

    // WRITING

    fn mknod(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        _rdev: u32,
        reply: ReplyEntry,
    ) {
        match self.fs_interface.make_inode(
            parent,
            name.to_string_lossy().to_string(),
            SimpleFileType::File,
        ) {
            Ok((id, _)) => {
                // creating metadata to return
                let mut new_attr = TEMPLATE_FILE_ATTR;
                new_attr.ino = id;
                new_attr.kind = FileType::RegularFile;
                new_attr.size = 0;

                reply.entry(&TTL, &new_attr, 0)
            }
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn mkdir(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        match self.fs_interface.make_inode(
            parent,
            name.to_string_lossy().to_string(),
            SimpleFileType::Directory,
        ) {
            Ok((id, _)) => {
                // creating metadata to return
                let mut new_attr = TEMPLATE_FILE_ATTR;
                new_attr.ino = id;
                new_attr.kind = FileType::Directory;
                new_attr.size = 0;

                reply.entry(&TTL, &new_attr, 0)
            }
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: fuser::ReplyEmpty) {
        match self.fs_interface.fuse_remove_inode(parent, name) {
            Ok(()) => reply.ok(),
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn rmdir(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: fuser::ReplyEmpty) {
        // should be only called on empty dirs ?
        match self.fs_interface.fuse_remove_inode(parent, name) {
            Ok(()) => reply.ok(),
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn rename(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        new_parent: u64,
        newname: &OsStr,
        _flags: u32,
        reply: fuser::ReplyEmpty,
    ) {
        match self.fs_interface.rename(
            parent,
            new_parent,
            &name //TODO move instead of ref because of the clone down the line
                .to_owned()
                .into_string()
                .expect("Don't support non unicode yet"), //TODO support OsString smartly
            &newname
                .to_owned()
                .into_string()
                .expect("Don't support non unicode yet"),
        ) {
            Ok(()) => reply.ok(),
            Err(err) => reply.error(err.raw_os_error().unwrap_or(EIO)),
        }
    }

    fn write(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyWrite,
    ) {
        let offset = offset
            .try_into()
            .expect("fuser write: can't convert i64 to u64");

        match self.fs_interface.write(ino, data.to_vec(), offset) {
            Ok(written) => reply.written(
                written
                    .try_into()
                    .expect("fuser write: can't convert u64 to u32"),
            ),
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    // ^ WRITING

    fn create(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: fuser::ReplyCreate,
    ) {
        match self.fs_interface.make_inode(
            parent,
            name.to_string_lossy().to_string(),
            SimpleFileType::File,
        ) {
            Ok((id, _)) => {
                // creating metadata to return
                let mut new_attr = TEMPLATE_FILE_ATTR;
                new_attr.ino = id;
                new_attr.kind = FileType::RegularFile;
                new_attr.size = 0;

                reply.created(&TTL, &new_attr, 0, new_attr.ino, flags as u32);
            }
            Err(err) => {
                log::error!("fuse_impl error: {:?}", err);
                reply.error(err.raw_os_error().unwrap_or(EIO))
            }
        }
    }

    fn open(&mut self, _req: &Request<'_>, ino: u64, flags: i32, reply: fuser::ReplyOpen) {
        reply.opened(ino, flags as u32); // TODO - check flags ?
    }

    fn release(
        &mut self,
        _req: &Request<'_>,
        _ino: u64,
        _fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: fuser::ReplyEmpty,
    ) {
        reply.ok();
    }
}

fn time_or_now_to_system_time(time: TimeOrNow) -> SystemTime {
    match time {
        TimeOrNow::Now => SystemTime::now(),
        TimeOrNow::SpecificTime(time) => time,
    }
}

pub fn mount_fuse(
    mount_point: &WhPath,
    fs_interface: Arc<FsInterface>,
) -> io::Result<BackgroundSession> {
    let options = vec![
        MountOption::RW,
        // MountOption::DefaultPermissions,
        MountOption::FSName("wormhole".to_string()),
    ];
    let ctrl = FuseController { fs_interface };

    fuser::spawn_mount2(ctrl, mount_point.to_string(), &options)
}
