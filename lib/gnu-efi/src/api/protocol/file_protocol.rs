
use super::Protocol;
use ::api::types::FunctionPointer;
use ::api::types::Guid;

#[repr(C)]
#[allow(non_snake_case)]
pub struct FileProtocol {
    revision: usize,
    Open:           extern fn(&mut FileProtocol, &mut *mut FileProtocol, *const u16, u64, u64) -> ::def::Status,
    Close:  FunctionPointer,
    Delete:     FunctionPointer,
    Read:           extern fn(&mut FileProtocol, &mut usize, *mut u8) -> ::def::Status,
    Write:  FunctionPointer,
    GetPosition:    FunctionPointer,
    SetPosition:    FunctionPointer,
    GetInfo:    FunctionPointer,
    SetInfo:    FunctionPointer,
    Flush:  FunctionPointer,
    OpenEx:     FunctionPointer,
    ReadEx:     FunctionPointer,
    WriteEx:    FunctionPointer,
    FlushEx:   FunctionPointer,
}

impl Protocol for FileProtocol {
    fn get_guid() -> Guid {
        ::api::types::FILE_GUID
    }
}

impl FileProtocol {
    pub fn open(&mut self, file_name: &str) -> Result<&mut FileProtocol, ::def::Status> {
        let mut result = 0 as *mut FileProtocol;

        let mut wide_file_name: [u16; 30] = [0; 30];

        for (i, c) in file_name.as_bytes().iter().enumerate() {
            wide_file_name[i] = *c as u16;
        }

        let status = ::bind::safe_efi_call5(
            self.Open,
            self,
            &mut result,
            wide_file_name.as_mut_ptr(),
            0x01,
            0x00);

        if status == ::def::Status::Success {
            unsafe {
                Ok(::core::mem::transmute(result))
            }
        } else {
            Err(status)
        }
    }

    pub fn read<'a>(&mut self, mut size: usize, buffer: &'a *mut u8) -> Result<&'a mut [u8], ::def::Status> {
        let status = ::bind::safe_efi_call3(
            self.Read,
            self,
            &mut size,
            *buffer);

        if status == ::def::Status::Success {
            unsafe {
                Ok(::core::slice::from_raw_parts_mut(*buffer, size))
            }
        } else {
            Err(status)
        }
    }
}
