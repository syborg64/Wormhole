use fuser::{FileAttr, FileType};
use log::info;
use std::{
    ffi::OsStr,
    fs::{self, create_dir, File},
    io::Write,
    path::{self, PathBuf},
};

use crate::network::message::{self, Folder, NetworkMessage};

use super::{Provider, TEMPLATE_FILE_ATTR};

impl Provider {
    // Basically no error handling for the poc
    // If good : Some(requested_data)
    // Else : None

    pub fn mkfile(&mut self, parent_ino: u64, name: &OsStr) -> Option<FileAttr> {
        // should check that the parent exists and is a folder
        // return None if error
        if let Some(meta) = self.get_metadata(parent_ino) {
            if meta.kind == FileType::Directory {
                let new_path =
                    PathBuf::from(self.mirror_path_from_inode(parent_ino).unwrap()).join(name);

                fs::File::create(&new_path).unwrap(); // real file creation

                // add entry to the index
                self.index.insert(
                    self.next_inode,
                    (
                        FileType::RegularFile,
                        new_path
                            .strip_prefix(self.local_source.clone())
                            .unwrap()
                            .to_string_lossy()
                            .to_string(),
                    ),
                );
                self.tx
                    .send(NetworkMessage::File(message::File {
                        path: new_path,
                        file: [].to_vec(),
                        ino: self.next_inode,
                    }))
                    .unwrap();
                let mut new_attr = TEMPLATE_FILE_ATTR;
                new_attr.ino = self.next_inode;
                new_attr.kind = FileType::RegularFile;
                new_attr.size = 0;
                self.next_inode += 1; // NOTE - ne jamais oublier d'incrémenter si besoin next_inode

                Some(new_attr)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn mkdir(&mut self, parent_ino: u64, name: &OsStr) -> Option<FileAttr> {
        // should check that the parent exists and is a folder
        // return None if error
        println!("Creating dir");
        if let Some(meta) = self.get_metadata(parent_ino) {
            if meta.kind == FileType::Directory {
                let new_path =
                    PathBuf::from(self.mirror_path_from_inode(parent_ino).unwrap()).join(name);
                println!("directory created: {:?}", new_path);
                fs::create_dir(&new_path).unwrap(); // create a real directory
                let virt_path = new_path
                    .strip_prefix(self.local_source.clone())
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                self.index
                    .insert(self.next_inode, (FileType::Directory, virt_path.clone()));
                self.tx
                    .send(NetworkMessage::NewFolder(Folder {
                        ino: self.next_inode,
                        path: virt_path.into(),
                    }))
                    .unwrap();
                let mut new_attr = TEMPLATE_FILE_ATTR;
                new_attr.ino = self.next_inode;
                new_attr.kind = FileType::Directory;
                new_attr.size = 0;
                self.next_inode += 1; // NOTE - ne jamais oublier d'incrémenter si besoin next_inode

                Some(new_attr)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn rmfile(&mut self, parent_ino: u64, name: &OsStr) -> Option<()> {
        // should only be called on files and not folders
        // if 404 or Folder -> None
        Some(())
    }

    pub fn rmdir(&mut self, parent_ino: u64, name: &OsStr) -> Option<()> {
        // should only be called on empty folders
        // if 404, not empty or file -> None
        Some(())
    }

    pub fn rename(
        &mut self,
        parent_ino: u64,
        name: &OsStr,
        newparent_ino: u64,
        newname: &OsStr,
    ) -> Option<()> {
        // pas clair de quand c'est appelé, si ça l'est sur des dossiers
        // non vides, go ignorer et pas tester à la démo
        Some(())
    }

    pub fn write(&self, ino: u64, offset: i64, data: &[u8]) -> Option<u32> {
        // returns the writed size
        info!("WRITE ENTERNAL");
        if let Some(path) = self.mirror_path_from_inode(ino) {
            let mut pos = 0;
            info!("WRITE GOOD PATH: {:?}", path);
            match File::options().append(true).open(path) {
                Ok(mut f) => {
                    info!("WRITE: {:?}", f);
                    while pos < data.len() {
                        if let Ok(bytes_written) = f.write(&data[pos..]) {
                            pos += bytes_written;
                        } else {
                            info!("WRITE ERROR");
                            return Some(pos as u32);
                        }
                    }
                    info!("WRITE {:?}", pos);
                    Some(pos as u32)
                }
                Err(_) => None,
            }
        } else {
            None
        }
        // Some(0)
    }

    // RECEPTION
    pub fn new_folder(&mut self, ino: u64, path: PathBuf) {
        println!("Provider make new folder");
        let real_path = PathBuf::from(self.local_source.clone()).join(&path);
        info!("AHAHA CREATING DIR with path {:?}", real_path);
        fs::create_dir(&real_path).unwrap();
        self.index.insert(
            ino,
            (FileType::Directory, path.to_string_lossy().to_string()),
        );
    }
}
