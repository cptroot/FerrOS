
use super::Protocol;
use super::file_protocol::FileProtocol;
use ::api::types::Guid;

#[repr(C)]
#[allow(non_snake_case)]
pub struct SimpleFileSystemProtocol {
    revision: usize,
    OpenVolume: extern fn(this:&mut SimpleFileSystemProtocol, root: &mut *mut FileProtocol) -> ::def::Status,
}

impl Protocol for SimpleFileSystemProtocol {
    fn get_guid() -> Guid {
        ::api::types::SIMPLE_FILE_SYSTEM_GUID
    }
}

impl SimpleFileSystemProtocol {
    pub fn open_volume(&mut self) -> Result<&mut FileProtocol, ::def::Status> {
        let mut root = 0 as *mut FileProtocol;
        let status = ::bind::safe_efi_call2(
            self.OpenVolume,
            self,
            &mut root);

        if status == ::def::Status::Success {
            unsafe {
                Ok(::core::mem::transmute(root))
            }
        } else {
            Err(status)
        }
    }
}
