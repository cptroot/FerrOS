
use super::Protocol;
use ::api::types::Guid;

#[repr(C)]
pub struct LoadedImageProtocol {
    revision: u32,
    parent_handle: ::def::Handle,
    system_table: *const ::api::SystemTable,

    // Source location of the image
    device_handle: ::def::Handle,
    pub file_path: &'static ::api::protocol::DevicePathProtocol,
    reserved: *const ::mem::c_void,

    // Image’s load options
    load_options_size: u32,
    load_options: *const ::mem::c_void,

    // Location where image was loaded
    image_base: *const ::mem::c_void,
    image_size: u64,
    image_code_type: ::def::MemoryType,
    image_data_type: ::def::MemoryType,
    unload: ::api::types::FunctionPointer,
}

impl Protocol for LoadedImageProtocol {
    fn get_guid() -> Guid {
        ::api::types::LOADED_IMAGE_PROTOCOL
    }
}
