
use super::Protocol;
use ::api::types::Guid;

#[repr(packed)]
pub struct DevicePathToTextProtocol {
    convert_device_node_to_text: extern fn(*const super::DevicePathProtocol, bool, bool) -> *const u16,
    convert_device_path_to_text: extern fn(*const super::DevicePathProtocol, bool, bool) -> *const u16,
}

impl Protocol for DevicePathToTextProtocol {
    fn get_guid() -> Guid {
        ::api::types::DEVICE_PATH_TO_TEXT_GUID
    }
}

impl DevicePathToTextProtocol {
    pub fn device_node_to_text(&self, device_node: &super::DevicePathProtocol) -> *const u16 {
        ::bind::safe_efi_call3(
            self.convert_device_node_to_text,
            device_node,
            false,
            false)
    }
    pub fn device_path_to_text(&self, device_path: &super::DevicePathProtocol) -> *const u16 {
        ::bind::safe_efi_call3(
            self.convert_device_path_to_text,
            device_path,
            false,
            false)
    }
}
