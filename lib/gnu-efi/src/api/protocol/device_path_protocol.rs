
use super::Protocol;
use ::api::types::Guid;

use core::mem::transmute;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Error {
    InvalidMainType,
    InvalidSubtype,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DevicePathType {
    HardwareDevicePath = 0x01,
    AcpiDevicePath = 0x02,
    MessagingDevicePath = 0x03,
    MediaDevice = 0x04,
    BiosBootSpecificationDevicePath = 0x05,
    EndOfHardwareDevicePath = 0x7F,
}

#[repr(packed)]
pub struct DevicePathProtocol {
    pub main_type: DevicePathType,
    subtype: u8,
    length: u16,
}

impl Protocol for DevicePathProtocol {
    fn get_guid() -> Guid {
        ::api::types::DEVICE_PATH_GUID
    }
}

impl<'a> IntoIterator for &'a DevicePathProtocol {
    type Item = &'a DevicePathProtocol;
    type IntoIter = DevicePathIterator<'a>;
    fn into_iter(self) -> Self::IntoIter {
        DevicePathIterator {
            iter: self,
        }
    }
}

pub struct DevicePathIterator<'a> {
    iter: &'a DevicePathProtocol,
}

impl<'a> Iterator for DevicePathIterator<'a> {
    type Item = &'a DevicePathProtocol;
    fn next(&mut self) -> Option<Self::Item> {
        if self.iter.main_type == DevicePathType::EndOfHardwareDevicePath {
            None
        } else {
            let result = self.iter;
            unsafe {
                let pointer: *const DevicePathProtocol = self.iter;
                let u8_pointer = pointer as *const u8;
                self.iter = transmute(u8_pointer.offset(self.iter.length as isize));
            }
            Some(result)
        }
    }
}

impl<'a> DevicePathIterator<'a> {
    pub fn new<'b>(pointer: &'b DevicePathProtocol) ->
            DevicePathIterator<'b> {
        DevicePathIterator {
            iter: pointer,
        }
    }
}

pub mod media_device_path {
    use core::convert::TryFrom;
    use core::slice::from_raw_parts;
    use core::mem::transmute;
    use super::{DevicePathProtocol, DevicePathType};
    use super::Error;

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum MediaDevicePathType {
        HardDrive           = 0x01,
        CdRom               = 0x02,
        Vendor              = 0x03,
        Filepath            = 0x04,
        MediaProtocol       = 0x05,
        PwigFirmwareFile    = 0x06,
        PwigFirmwareVolume  = 0x07,
        RelativeOffsetRange = 0x08,
        RamDisk             = 0x09,
    }

    #[repr(packed)]
    pub struct FilepathDevicePath {
        pub main_type: DevicePathType,
        pub subtype: MediaDevicePathType,
        pub length: u16,
        pub path_name: [u16],
    }

    impl<'a> TryFrom<&'a DevicePathProtocol> for &'a FilepathDevicePath {
        type Error = Error;
        fn try_from(device_path: &'a DevicePathProtocol) -> Result<&'a FilepathDevicePath, Self::Error> {
            if device_path.main_type == DevicePathType::MediaDevice {
                if device_path.subtype == MediaDevicePathType::Filepath as u8 {
                    unsafe {
                        let pointer = (device_path as *const DevicePathProtocol) as *const u8;
                        let array: &[u8] = from_raw_parts(pointer, device_path.length as usize);
                        Ok(transmute(array))
                    }
                } else {
                    Err(Error::InvalidSubtype)
                }
            } else {
                Err(Error::InvalidMainType)
            }
        }
    }
}

pub mod device_path_end {
    use core::convert::TryFrom;
    use core::mem::transmute;
    use super::{DevicePathProtocol, DevicePathType};
    use super::Error;

    #[repr(u8)]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum DevicePathEndType {
        EndEntireDevicePath = 0xFF,
        EndDevicePathInstance = 0x01,
    }

    #[repr(packed)]
    pub struct DevicePathEnd {
        pub main_type: DevicePathType,
        pub subtype: DevicePathEndType,
        pub length: u16,
    }

    impl<'a> TryFrom<&'a DevicePathProtocol> for &'a DevicePathEnd {
        type Error = Error;
        fn try_from(device_path: &'a DevicePathProtocol) -> Result<&'a DevicePathEnd, Self::Error> {
            if device_path.main_type == DevicePathType::EndOfHardwareDevicePath {
                if device_path.subtype == DevicePathEndType::EndEntireDevicePath as u8 ||
                    device_path.subtype == DevicePathEndType::EndDevicePathInstance as u8 {
                    unsafe {
                        Ok(transmute(device_path))
                    }
                } else {
                    Err(Error::InvalidSubtype)
                }
            } else {
                Err(Error::InvalidMainType)
            }
        }
    }

}
