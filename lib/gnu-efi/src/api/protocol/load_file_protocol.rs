
use super::Protocol;
use super::device_path_protocol::DevicePathProtocol;
use ::api::types::Guid;

#[repr(packed)]
#[allow(non_snake_case)]
pub struct LoadFileProtocol {
    LoadFile: extern fn(this: &mut LoadFileProtocol, file_path: &DevicePathProtocol, boot_policy: bool, buffer_size: &mut usize, buffer: *mut ::mem::c_void) -> ::def::Status,
}

impl Protocol for LoadFileProtocol {
    fn get_guid() -> Guid {
        ::api::types::LOAD_FILE_GUID
    }
}

impl LoadFileProtocol {
    pub fn load_file<'a>(&mut self, device_path: &DevicePathProtocol) -> Result<&'a [u8], ::def::Status> {
        let buffer = 0 as *mut ::mem::c_void;
        let mut size: usize = 40000;
        let status = ::bind::safe_efi_call5(
            self.LoadFile,
            self,
            device_path,
            false,
            &mut size,
            buffer);

        if status == ::def::Status::Success {
            unsafe {
                Ok(::core::slice::from_raw_parts(buffer as *const u8, size))
            }
        } else {
            Err(status)
        }
    }
}
