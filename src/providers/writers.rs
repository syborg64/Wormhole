use std::{
    ffi::OsStr,
    fs::{self, create_dir, File},
    io::Write,
    path::{self, PathBuf},
};

use fuser::{FileAttr, FileType};
use log::info;

use crate::network::message::{File, Folder, NetworkMessage};

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
                        new_path.to_string_lossy().to_string(),
                    ),
                );
                self.tx
                    .send(NetworkMessage::File(File {
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
                self.index.insert(
                    self.next_inode,
                    (FileType::Directory, new_path.to_string_lossy().to_string()),
                );
                self.tx
                    .send(NetworkMessage::NewFolder(Folder {
                        ino: self.next_inode,
                        path: new_path,
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

        // if let Some(parent_data) = self.get_metadata(parent_ino) {
        //     if parent_data.kind == FileType::Directory {
        //         if let Some(meta_folder) = self.fs_lookup(parent_ino, name) {
        //             if meta_folder.kind == FileType::Directory {
        //                 if let Some(inode_list) = self.list_files(parent_ino) {
        //                     if inode_list.is_empty() {
        //                         let path =
        //                             PathBuf::from(self.mirror_path_from_inode(parent_ino).unwrap())
        //                                 .join(name);
        //                         println!("The directory {:?} is deleted", path);
        //                         fs::remove_dir(path).unwrap();
        //                         Some(())
        //                     } else {
        //                         None
        //                     }
        //                 } else {
        //                     None
        //                 }
        //             } else {
        //                 None
        //             }
        //         } else {
        //             None
        //         }
        //     } else {
        //         None
        //     }
        // } else {
        //     None
        // }
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
        // if let Some(path) = self.mirror_path_from_inode(ino) {
        //     let mut f = File::options().append(true).open(path);
        //     let mut pos = 0;
        //     while pos < data.len() {
        //         match f.write(&data[pos..]) {
        //             Ok(bytes) => pos += bytes;

        //         }
        //     }
        //     Some(pos as u32)
        // } else {
        //     None
        // }
    }

    // RECEPTION
    pub fn new_folder(&mut self, ino: u64, path: PathBuf) {
        println!("Provider make new folder");
        fs::create_dir(&path).unwrap();
        self.index.insert(
            ino,
            (FileType::Directory, path.to_string_lossy().to_string()),
        );
    }
}
