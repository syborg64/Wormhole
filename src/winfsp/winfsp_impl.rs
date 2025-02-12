use std::{cmp::min, ffi::OsString, io::ErrorKind, sync::Arc};

use ntapi::ntioapi::FILE_DIRECTORY_FILE;
use winapi::{
    shared::{
        ntstatus::{STATUS_INVALID_DEVICE_REQUEST, STATUS_SUCCESS},
        winerror::{
            ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND, ERROR_GEN_FAILURE, ERROR_INVALID_NAME,
            ERROR_NOT_FOUND,
        },
    },
    um::winnt::{
        FILE_ALL_ACCESS, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_NORMAL, FILE_READ_ATTRIBUTES,
        FILE_WRITE_ATTRIBUTES,
    },
};
use windows::Win32::Foundation::{NTSTATUS, WIN32_ERROR};
use winfsp::{
    filesystem::{DirInfo, FileInfo, FileSecurity, FileSystemContext, WideNameInfo}, host::{FileSystemHost, VolumeParams}, U16CStr, U16CString
};
use winfsp_sys::FILE_ACCESS_RIGHTS;

use crate::pods::{
    arbo::Arbo,
    fs_interface::{FsInterface, SimpleFileType},
    whpath::WhPath,
};

impl TryInto<WhPath> for &winfsp::U16CStr {
    type Error = WIN32_ERROR;

    fn try_into(self) -> Result<WhPath, Self::Error> {
        match self.to_string() {
            Err(_) => Err(WIN32_ERROR(ERROR_INVALID_NAME)),
            Ok(string) => Ok(WhPath::from(&string.replace("\\", "/"))),
        }
    }
}

impl WhPath {
    pub fn to_winfsp(&self) -> String {
        self.inner.replace("/", "\\")
    }
}

#[derive(PartialEq, Debug)]
pub struct WormholeHandle(u64);

pub struct FSPController {
    pub fs_interface: Arc<FsInterface>,
    pub dummy_file: OsString,
    // pub provider: Arc<RwLock<Provider<WindowsFolderHandle>>>,
}

pub fn mount_fsp(
    path: &WhPath,
    fs_interface: Arc<FsInterface>,
) -> Result<FileSystemHost<'static>, std::io::Error> {
    let volume_params = VolumeParams::default();

    println!("created volume params...");
    let wormhole_context = FSPController {
        fs_interface,
        dummy_file: "dummy".into(), // dummy_file: (&path.clone().rename(&("dummy_file").to_string()).inner).into(),
    };
    println!("creating host...");
    let mut host = FileSystemHost::new::<FSPController>(volume_params, wormhole_context)
        .map_err(|_| std::io::Error::new(ErrorKind::Other, "oh no!"))?;
    // .expect("FSHost::new");
    println!("created host...");

    println!("mounting host...");
    let _ = host.mount(&path.inner)?; //expect("winfsp: host.mount");
                                      // let result = host.mount("./winfsp_mount")?;
    println!("mounted host...");
    host.start_with_threads(1)?;
    println!("started host...");
    Ok(host)
}

impl FileSystemContext for FSPController {
    type FileContext = WormholeHandle;

    fn get_security_by_name(
        &self,
        file_name: &winfsp::U16CStr,
        security_descriptor: Option<&mut [std::ffi::c_void]>,
        reparse_point_resolver: impl FnOnce(
            &winfsp::U16CStr,
        ) -> Option<winfsp::filesystem::FileSecurity>,
    ) -> winfsp::Result<winfsp::filesystem::FileSecurity> {
        // thread::sleep(std::time::Duration::from_secs(2));
        log::info!(
            "winfsp::get_security_by_name({}, {:?})",
            file_name.to_string_lossy(),
            security_descriptor.as_ref().map(|s| s.len())
        );
        
        if let Some(security) = reparse_point_resolver(file_name) {
            return Ok(security);
        }

        // return Err(winfsp::FspError::WIN32(ERROR_NOT_FOUND));

        //let arbo: &Arbo = &self.fs_interface.arbo.read();
        //let arbo = Arbo::read_lock(&self.fs_interface.arbo, "get_security_by_name")?;

        let path: WhPath = file_name
            .try_into()
            .inspect_err(|e| log::error!("{}:{:?}", file_name.to_string_lossy(), e))?;

        let file_type: SimpleFileType = (&Arbo::read_lock(&self.fs_interface.arbo, "get_security_by_name")?
        .get_inode_from_path(&path)
        .inspect_err(|e| log::error!("{}:{:?}", &path.inner, e))?.entry).into();
        // let mut descriptor_size = 0;
        // let option_sd = if security_descriptor.is_some() {
        //     Some(
        //         self.dummy_file
        //             .security_descriptor(SecurityInformation::all()).map_err(|e| {log::error!("{}:{:?}", &self.dummy_file.to_string_lossy(), e); e})?
        //     )
        // } else {
        //     None
        // };
        // if let (Some(sec_dec_storage), Some(got_sd)) = (security_descriptor, option_sd) {
        //     descriptor_size = unsafe {
        //         winapi::um::securitybaseapi::GetSecurityDescriptorLength(got_sd.as_ptr() as *mut _)
        //     } as usize;

        //     if sec_dec_storage.len() >= descriptor_size {
        //         unsafe {
        //             (got_sd.as_ptr() as *mut u8)
        //                 .copy_to(sec_dec_storage.as_ptr() as *mut u8, descriptor_size);
        //         }
        //     };
        // }
        let mut attributes: u32 = FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES;
        attributes |= match file_type {
            SimpleFileType::File => FILE_ATTRIBUTE_NORMAL,
            SimpleFileType::Directory => FILE_ATTRIBUTE_DIRECTORY,
            // SimpleFileType::Link => FILE_ATTRIBUTE_REPARSE_POINT,
            // SimpleFileType::Other => FILE_ATTRIBUTE_READONLY, // TODO: remove ?
        };
        let sec = FileSecurity {
            reparse: false,
            sz_security_descriptor: 0,
            // sz_security_descriptor: descriptor_size as u64,
            attributes,
        };
        log::info!("ok({:?})", sec);
        winfsp::Result::Ok(sec)
    }

    fn open(
        &self,
        file_name: &winfsp::U16CStr,
        _create_options: u32,
        _granted_access: FILE_ACCESS_RIGHTS,
        file_info: &mut winfsp::filesystem::OpenFileInfo,
    ) -> winfsp::Result<Self::FileContext> {
        // thread::sleep(std::time::Duration::from_secs(2));
        log::info!("winfsp::open({})", file_name.display());

        let path: WhPath = file_name.try_into().map_err(|e| {
            log::error!("{:?}", e);
            e
        })?;
        return match Arbo::read_lock(&self.fs_interface.arbo, "winfsp::open")?.get_inode_from_path(&path) {
            Ok(inode) => {
                let mut attributes: u32 = FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES;
                attributes |= match (&inode.entry).into() {
                    SimpleFileType::File => FILE_ATTRIBUTE_NORMAL,
                    SimpleFileType::Directory => FILE_ATTRIBUTE_DIRECTORY,
                    // SimpleFileType::Link => FILE_ATTRIBUTE_REPARSE_POINT,
                    // SimpleFileType::Other => FILE_ATTRIBUTE_READONLY, // TODO: remove ?
                };
                *file_info.as_mut() = FileInfo {
                    file_attributes: attributes,
                    reparse_tag: 0,
                    allocation_size: 0,
                    file_size: 0,
                    creation_time: 0,
                    last_access_time: 0,
                    last_write_time: 0,
                    change_time: 0,
                    index_number: inode.id,
                    hard_links: 0,
                    ea_size: 0,
                };
                file_info.set_normalized_name(file_name.as_slice(), None);
                // file_info.set_normalized_name(U16CString::from_str(&inode.name).expect("u16str from str").as_slice(), None);
                log::info!("ok:{}", inode.id);
                Ok(WormholeHandle(inode.id))
            }
            Err(err) => {
                log::error!("{:?}", err);
                if err.kind() == ErrorKind::NotFound {
                    Err(winfsp::FspError::WIN32(ERROR_FILE_NOT_FOUND))
                } else {
                    Err(winfsp::FspError::WIN32(ERROR_GEN_FAILURE))
                }
            }
        };
    }

    fn close(&self, context: Self::FileContext) {
        // thread::sleep(std::time::Duration::from_secs(2));
        log::info!("winfsp::close({:?})", context);

    }

    fn create(
        &self,
        file_name: &winfsp::U16CStr,
        create_options: u32,
        granted_access: FILE_ACCESS_RIGHTS,
        file_attributes: winfsp_sys::FILE_FLAGS_AND_ATTRIBUTES,
        security_descriptor: Option<&[std::ffi::c_void]>,
        allocation_size: u64,
        extra_buffer: Option<&[u8]>,
        extra_buffer_is_reparse_point: bool,
        file_info: &mut winfsp::filesystem::OpenFileInfo,
    ) -> winfsp::Result<Self::FileContext> {
        let file_type = match (create_options & FILE_DIRECTORY_FILE) != 0 {
            true => SimpleFileType::Directory,
            false => SimpleFileType::File,
        };
        // thread::sleep(std::time::Duration::from_secs(2));
        log::info!("winfsp::create({:?}, type: {:?})", file_name, file_type);

        let path: WhPath = file_name.try_into()?;
        let (folder, name) = path.split_folder_file();
        let arbo = Arbo::write_lock(&self.fs_interface.arbo, "winfsp::create")?;

        if let Ok(_) = arbo.get_inode_from_path(&path) {
            return Err(winfsp::FspError::WIN32(ERROR_ALREADY_EXISTS));
        }

        let parent = arbo
            .get_inode_from_path(&(&folder).into())
            .map_err(|_| winfsp::FspError::WIN32(ERROR_NOT_FOUND))?.id;

        drop(arbo);

        let mut attributes: u32 = FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES;
        attributes |= match file_type {
            SimpleFileType::File => FILE_ATTRIBUTE_NORMAL,
            SimpleFileType::Directory => FILE_ATTRIBUTE_DIRECTORY,
            // SimpleFileType::Link => FILE_ATTRIBUTE_REPARSE_POINT,
            // SimpleFileType::Other => FILE_ATTRIBUTE_READONLY, // TODO: remove ?
        };
        let inode = self
            .fs_interface
            .make_inode(parent, name, file_type)
            .inspect_err(|e| log::error!("make_inode:{e}"))?;
        *file_info.as_mut() = FileInfo {
            file_attributes: attributes,
            reparse_tag: 0,
            allocation_size: 0,
            file_size: 0,
            creation_time: 0,
            last_access_time: 0,
            last_write_time: 0,
            change_time: 0,
            index_number: inode.0,
            hard_links: 0,
            ea_size: 0,
        };
        file_info.set_normalized_name(file_name.as_slice(), None);

        Ok(WormholeHandle(inode.0))
    }

    fn cleanup(
        &self,
        context: &Self::FileContext,
        file_name: Option<&winfsp::U16CStr>,
        flags: u32,
    ) {
    }

    fn flush(
        &self,
        context: Option<&Self::FileContext>,
        file_info: &mut winfsp::filesystem::FileInfo,
    ) -> winfsp::Result<()> {
        Ok(())
        //         Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    }

    fn get_file_info(
        &self,
        context: &Self::FileContext,
        file_info: &mut winfsp::filesystem::FileInfo,
    ) -> winfsp::Result<()> {
        log::info!("winfsp::get_file_info({:?})", context);

        let arbo = Arbo::read_lock(&self.fs_interface.arbo, "winfsp::get_file_info")?;
        let path = arbo.get_path_from_inode_id(context.0)?;

        return match arbo.get_inode(context.0) {
            Ok(inode) => {
                let mut attributes: u32 = FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES;
                attributes |= match (&inode.entry).into() {
                    SimpleFileType::File => FILE_ATTRIBUTE_NORMAL,
                    SimpleFileType::Directory => FILE_ATTRIBUTE_DIRECTORY,
                    // SimpleFileType::Link => FILE_ATTRIBUTE_REPARSE_POINT,
                    // SimpleFileType::Other => FILE_ATTRIBUTE_READONLY, // TODO: remove ?
                };
                *file_info = FileInfo {
                    file_attributes: attributes,
                    reparse_tag: 0,
                    allocation_size: 0,
                    file_size: 0,
                    creation_time: 0,
                    last_access_time: 0,
                    last_write_time: 0,
                    change_time: 0,
                    index_number: inode.id,
                    hard_links: 0,
                    ea_size: 0,
                };
                log::info!("ok:{:?}", file_info);
                Ok(())
            }
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    Err(winfsp::FspError::WIN32(ERROR_FILE_NOT_FOUND))
                } else {
                    Err(winfsp::FspError::WIN32(ERROR_GEN_FAILURE))
                }
            }
        };
    }

    fn get_security(
        &self,
        context: &Self::FileContext,
        security_descriptor: Option<&mut [std::ffi::c_void]>,
    ) -> winfsp::Result<u64> {
        log::info!("winfsp::get_security({:?})", context);

        Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    }

    // fn set_security(
    //     &self,
    //     context: &Self::FileContext,
    //     security_information: u32,
    //     modification_descriptor: winfsp::filesystem::ModificationDescriptor,
    // ) -> winfsp::Result<()> {
    //     Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    // }

    // fn overwrite(
    //     &self,
    //     context: &Self::FileContext,
    //     file_attributes: winfsp_sys::FILE_FLAGS_AND_ATTRIBUTES,
    //     replace_file_attributes: bool,
    //     allocation_size: u64,
    //     extra_buffer: Option<&[u8]>,
    //     file_info: &mut winfsp::filesystem::FileInfo,
    // ) -> winfsp::Result<()> {
    //     Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    // }

    fn read_directory(
        &self,
        context: &Self::FileContext,
        pattern: Option<&winfsp::U16CStr>,
        marker: winfsp::filesystem::DirMarker,
        buffer: &mut [u8],
    ) -> winfsp::Result<u32> {
        // thread::sleep(std::time::Duration::from_secs(2));
        log::info!("winfsp::read_directory({:?}, marker: {:?})", context, marker.inner_as_cstr().map(|s|s.to_string_lossy()));
        // return Ok(STATUS_SUCCESS as u32);
        let mut entries = if let Ok(entries) = self.fs_interface.read_dir(context.0) {
            entries
        } else {
            log::error!("err:{ERROR_NOT_FOUND}");
            return Err(WIN32_ERROR(ERROR_NOT_FOUND).into());
        };

        let mut cursor = 0;

        entries.sort_by(|a, b| a.name.cmp(&b.name));
        let marker = marker.inner_as_cstr().map(|s| s.to_string_lossy());
        for entry in entries
            .into_iter()
            .skip_while(|s| marker.as_ref().map(|m| &s.name <= m).unwrap_or(false))
        {
            let mut dirinfo = DirInfo::<255>::default(); // !todo
                                                         // let mut info = dirinfo.file_info_mut();
            dirinfo.set_name(&entry.name)?;

            let mut attributes: u32 = FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES;
            attributes |= match (&entry.entry).into() {
                SimpleFileType::File => FILE_ATTRIBUTE_NORMAL,
                SimpleFileType::Directory => FILE_ATTRIBUTE_DIRECTORY,
                // SimpleFileType::Link => FILE_ATTRIBUTE_REPARSE_POINT,
                // SimpleFileType::Other => FILE_ATTRIBUTE_READONLY, // TODO: remove ?
            };
            *dirinfo.file_info_mut() = FileInfo {
                file_attributes: attributes,
                reparse_tag: 0,
                allocation_size: 0,
                file_size: 0,
                creation_time: 0,
                last_access_time: 0,
                last_write_time: 0,
                change_time: 0,
                index_number: entry.id,
                hard_links: 0,
                ea_size: 0,
            };

            log::info!("dirinfo:{}:{:?}", &entry.name, dirinfo.file_info_mut());
            if !dirinfo.append_to_buffer(buffer, &mut cursor) {
                break;
            }
        }
        DirInfo::<255>::finalize_buffer(buffer, &mut cursor);
        log::info!("ok:{cursor}");
        Ok(cursor as u32)
        // Ok(STATUS_SUCCESS as u32)
    }

    // fn rename(
    //     &self,
    //     context: &Self::FileContext,
    //     file_name: &winfsp::U16CStr,
    //     new_file_name: &winfsp::U16CStr,
    //     replace_if_exists: bool,
    // ) -> winfsp::Result<()> {
    //     Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    // }

    fn set_basic_info(
        &self,
        context: &Self::FileContext,
        file_attributes: u32,
        creation_time: u64,
        last_access_time: u64,
        last_write_time: u64,
        last_change_time: u64,
        file_info: &mut winfsp::filesystem::FileInfo,
    ) -> winfsp::Result<()> {
        Ok(())
        // Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    }

    fn set_delete(
        &self,
        context: &Self::FileContext,
        file_name: &winfsp::U16CStr,
        delete_file: bool,
    ) -> winfsp::Result<()> {

        Ok(())
        // Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    }

    fn set_file_size(
        &self,
        context: &Self::FileContext,
        new_size: u64,
        set_allocation_size: bool,
        file_info: &mut winfsp::filesystem::FileInfo,
    ) -> winfsp::Result<()> {
        Ok(())
        // Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    }

    fn read(
        &self,
        context: &Self::FileContext,
        buffer: &mut [u8],
        offset: u64,
    ) -> winfsp::Result<u32> {
        log::info!("winfsp::read({:?})", context);
        let data = self
            .fs_interface
            .read_file(context.0, offset, buffer.len() as u64)?;
        let len = min(data.len(), buffer.len());
        buffer[0..len].copy_from_slice(&data[0..len]);
        Ok(len as u32)
    }

    fn write(
        &self,
        context: &Self::FileContext,
        buffer: &[u8],
        offset: u64,
        write_to_eof: bool,
        constrained_io: bool,
        file_info: &mut winfsp::filesystem::FileInfo,
    ) -> winfsp::Result<u32> {
        log::info!("winfsp::write({:?})", context);
        let size = self.fs_interface.write(context.0, buffer, offset)?;
        Ok(size as u32)
    }

    // fn get_dir_info_by_name(
    //     &self,
    //     context: &Self::FileContext,
    //     file_name: &winfsp::U16CStr,
    //     out_dir_info: &mut winfsp::filesystem::DirInfo,
    // ) -> winfsp::Result<()> {
    //     log::info!("winfsp::get_dir_info_by_name({:?})", context);

    //     Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    // }

    // fn get_volume_info(
    //     &self,
    //     out_volume_info: &mut winfsp::filesystem::VolumeInfo,
    // ) -> winfsp::Result<()> {
    //     log::info!("winfsp::get_volume_info");
    //     Ok(())
    //     // Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    // }

    // fn set_volume_label(
    //     &self,
    //     volume_label: &winfsp::U16CStr,
    //     volume_info: &mut winfsp::filesystem::VolumeInfo,
    // ) -> winfsp::Result<()> {
    //     Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    // }

    // fn get_stream_info(
    //     &self,
    //     context: &Self::FileContext,
    //     buffer: &mut [u8],
    // ) -> winfsp::Result<u32> {
    //     log::info!("winfsp::get_stream_info({:?})", context);
    //     Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    // }

    // fn get_reparse_point_by_name(
    //     &self,
    //     file_name: &winfsp::U16CStr,
    //     is_directory: bool,
    //     buffer: &mut [u8],
    // ) -> winfsp::Result<u64> {
    //     log::info!("winfsp::get_reparse_point_by_name({:?})", file_name);

    //     Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    // }

    // fn get_reparse_point(
    //     &self,
    //     context: &Self::FileContext,
    //     file_name: &winfsp::U16CStr,
    //     buffer: &mut [u8],
    // ) -> winfsp::Result<u64> {
    //     log::info!("winfsp::get_reparse_point({:?})", context);

    //     Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    // }

    // fn set_reparse_point(
    //     &self,
    //     context: &Self::FileContext,
    //     file_name: &winfsp::U16CStr,
    //     buffer: &[u8],
    // ) -> winfsp::Result<()> {
    //     Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    // }

    // fn delete_reparse_point(
    //     &self,
    //     context: &Self::FileContext,
    //     file_name: &winfsp::U16CStr,
    //     buffer: &[u8],
    // ) -> winfsp::Result<()> {
    //     Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    // }

    // fn get_extended_attributes(
    //     &self,
    //     context: &Self::FileContext,
    //     buffer: &mut [u8],
    // ) -> winfsp::Result<u32> {
    //     Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    // }

    // fn set_extended_attributes(
    //     &self,
    //     context: &Self::FileContext,
    //     buffer: &[u8],
    //     file_info: &mut winfsp::filesystem::FileInfo,
    // ) -> winfsp::Result<()> {
    //     Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    // }

    fn control(
        &self,
        context: &Self::FileContext,
        control_code: u32,
        input: &[u8],
        output: &mut [u8],
    ) -> winfsp::Result<u32> {
        Err(NTSTATUS(STATUS_INVALID_DEVICE_REQUEST).into())
    }

    fn dispatcher_stopped(&self, normally: bool) {}

    unsafe fn with_operation_response<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&mut winfsp_sys::FSP_FSCTL_TRANSACT_RSP) -> T,
    {
        unsafe {
            if let Some(context) = winfsp_sys::FspFileSystemGetOperationContext().as_ref() {
                if let Some(response) = context.Response.as_mut() {
                    return Some(f(response));
                }
            }
        }
        None
    }

    unsafe fn with_operation_request<T, F>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&winfsp_sys::FSP_FSCTL_TRANSACT_REQ) -> T,
    {
        unsafe {
            if let Some(context) = winfsp_sys::FspFileSystemGetOperationContext().as_ref() {
                if let Some(request) = context.Request.as_ref() {
                    return Some(f(request));
                }
            }
        }
        None
    }
}
