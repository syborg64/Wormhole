use fuser::{
    BackgroundSession, FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyData,
    ReplyDirectory, ReplyEntry, Request,
};
use libc::{ENOENT, ENOSYS};
use log::debug;
use openat::{Dir, SimpleType};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::mpsc::UnboundedSender;
use walkdir::WalkDir;

use crate::network::message::NetworkMessage;
use crate::providers::{FsIndex, Provider};

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

const TEMPLATE_FILE_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 13,
    blocks: 1,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
    blksize: 512,
};
// ^ placeholders

// const MIRROR_PTH: &str = "./wh_mirror/";

pub struct FuseController {
    pub provider: Arc<Mutex<Provider>>,
}

// Custom data (not directly linked to fuse)
// create some data for us like the index or data provider

fn simple_type_to_fuse_type(t: SimpleType) -> fuser::FileType {
    match t {
        SimpleType::Dir => fuser::FileType::Directory,
        SimpleType::File => fuser::FileType::RegularFile,
        _ => fuser::FileType::CharDevice, // NOTE - random because unsupported, should be handled
    }
}
fn index_folder_recursive(
    arbo: &mut FsIndex,
    inode: &mut u64,
    root_fd: &Dir,
    path: PathBuf,
) -> io::Result<()> {
    let errors_nb = root_fd
        .list_dir(&path)?
        .map(|entry| -> io::Result<()> {
            let entry = entry?;

            let name = entry.file_name();
            let stype = entry.simple_type().unwrap();

            arbo.insert(
                *inode,
                (simple_type_to_fuse_type(stype), path.clone().join(name)),
            );
            println!("added entry to arbo {}:{:?}", inode, arbo.get(inode));
            *inode += 1;

            if stype == SimpleType::Dir {
                index_folder_recursive(arbo, inode, root_fd, path.clone().join(name))?;
            }
            Ok(())
        })
        .filter(|e| e.is_err())
        .collect::<Vec<Result<(), io::Error>>>()
        .len();
    println!(
        "indexing: {} error(s) in folder {}",
        errors_nb,
        path.display()
    );
    Ok(())
}
impl FuseController {
    fn index_folder2(path: PathBuf) -> io::Result<FsIndex> {
        let metal_mount_handle = Dir::open(&path)?;
        let mut arbo: FsIndex = HashMap::new();
        let mut inode: u64 = 2;

        index_folder_recursive(&mut arbo, &mut inode, &metal_mount_handle, ".".into());
        Ok(arbo)
    }
    // we create an index of the original folder
    fn index_folder(source: &Path) -> FsIndex {
        println!("calling new version");
        let pouet = Self::index_folder2(PathBuf::from(&source));
        println!("end with\n{:?}\n_______", pouet);
        let mut arbo: FsIndex = HashMap::new();
        let mut inode: u64 = 2;

        arbo.insert(1, (fuser::FileType::Directory, "".into()));

        for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
            let strpath = entry.path();
            let path_type = if entry.file_type().is_dir() {
                fuser::FileType::Directory
            } else if entry.file_type().is_file() {
                fuser::FileType::RegularFile
            } else {
                fuser::FileType::CharDevice // random to detect unsupported
            };

            if *strpath != *source && path_type != fuser::FileType::CharDevice {
                let relative_path = PathBuf::from(&strpath)
                    .strip_prefix(source)
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                debug!("indexing {:?} to {}", strpath, relative_path);
                arbo.insert(inode, (path_type, relative_path.into()));
                inode += 1;
            } else {
                debug!("ignoring {:?}", strpath);
            }
        }
        arbo
    }
}

// REVIEW - should later invest in proper error handling
impl Filesystem for FuseController {
    // READING

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        debug!("lookup is called {} {:?}", parent, name);
        let provider = self.provider.lock().unwrap();
        if let Ok(file_attr) = provider.fs_lookup(parent, name) {
            reply.entry(&TTL, &file_attr, 0)
        } else {
            reply.error(ENOENT)
        }
    }

    // TODO
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        debug!("getattr is called {}", ino);
        match ino {
            1 => reply.attr(&TTL, &MOUNT_DIR_ATTR),
            2 => reply.attr(&TTL, &TEMPLATE_FILE_ATTR),
            _ => reply.error(ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        _size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        debug!("read is called");
        let provider = self.provider.lock().unwrap();
        if let Ok(content) = provider.read(ino) {
            reply.data(&content[offset as usize..])
        } else {
            reply.error(ENOENT);
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
        debug!("readdir is called for ino {}", ino);
        let provider = self.provider.lock().unwrap();
        if let Ok(entries) = provider.fs_readdir(ino) {
            for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
                debug!("readdir entries : {:?}", entry);
                // i + 1 means the index of the next entry
                if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                    break;
                }
            }
            reply.ok()
        } else {
            debug!("readdir EONENT ");
            reply.error(ENOENT)
        }
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
        let mut provider = self.provider.lock().unwrap();
        if let Ok(attr) = provider.mkfile(parent, name) {
            reply.entry(&TTL, &attr, 0)
        } else {
            reply.error(ENOSYS)
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
        let mut provider = self.provider.lock().unwrap();
        if let Ok(attr) = provider.mkdir(parent, name) {
            reply.entry(&TTL, &attr, 0)
        } else {
            reply.error(ENOSYS)
        }
    }

    fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: fuser::ReplyEmpty) {
        let mut provider = self.provider.lock().unwrap();
        if let Ok(()) = provider.rmfile(parent, name) {
            reply.ok()
        } else {
            reply.error(ENOENT)
        }
    }

    fn rmdir(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: fuser::ReplyEmpty) {
        // should be only called on empty dirs ?
        let mut provider = self.provider.lock().unwrap();
        if let Some(()) = provider.rmdir(parent, name) {
            reply.ok()
        } else {
            reply.error(ENOENT)
        }
    }

    fn rename(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        _flags: u32,
        reply: fuser::ReplyEmpty,
    ) {
        // comment sont gérés les dossiers et sous fichiers ?
        let mut provider = self.provider.lock().unwrap();
        if let Some(()) = provider.rename(parent, name, newparent, newname) {
            reply.ok()
        } else {
            reply.error(ENOENT)
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
        let provider = self.provider.lock().unwrap();
        if let Ok(written) = provider.write(ino, offset, data) {
            reply.written(written)
        } else {
            reply.error(ENOENT)
        }
    }

    // ^ WRITING
}

pub fn mount_fuse(
    source: &Path,
    mountpoint: &Path,
    tx: UnboundedSender<NetworkMessage>,
) -> (BackgroundSession, Arc<Mutex<Provider>>) {
    let options = vec![MountOption::RW, MountOption::FSName("wormhole".to_string())];
    let index = FuseController::index_folder(source);
    println!("FUSE MOUNT, actual file index:\n{:#?}", index);
    let provider = Arc::new(Mutex::new(Provider {
        next_inode: (index.len() + 2) as u64,
        index,
        local_source: source.to_path_buf(),
        tx,
    }));
    let ctrl = FuseController {
        provider: provider.clone(),
    };
    (
        fuser::spawn_mount2(ctrl, mountpoint, &options).unwrap(),
        provider,
    )
}
