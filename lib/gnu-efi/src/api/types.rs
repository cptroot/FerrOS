
use ::def;
use core::slice;

pub use super::services::{RuntimeServices, BootServices};

/// Types used by the efiapi.h file

/// Header for all UEFI tables.
#[repr(C)]
pub struct TableHeader {
    signature: u64,
    revision: u32,
    header_size: u32,
    crc32: u32,
    reserved: u32,
}

/// Parameter to ResetSystem
#[repr(usize)]
#[derive(Clone, Copy)]
pub enum ResetType {
    ResetCold,
    ResetWarm,
    ResetShutdown,
    ResetPlatformSpecific,
}

/// Parameter to LocateHandle
#[repr(usize)]
#[derive(Clone, Copy)]
pub enum LocateSearchType {
    AllHandles,
    ByRegisterNotify,
    ByProtocol,
}

/// Guid used by the UEFI vendor table
/// Contains things like ACPI table info
#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

pub const _ACPI_TABLE_GUID: Guid = Guid {
    data1: 0xeb9d2d30,
    data2: 0x2d88,
    data3: 0x11d3,
    data4: [0x9a, 0x16, 0x0, 0x90, 0x27, 0x3f, 0xc1, 0x4d],
};

pub const ACPI_20_TABLE_GUID: Guid = Guid {
    data1: 0x8868e871,
    data2: 0xe4f1,
    data3: 0x11d3,
    data4: [0xbc, 0x22, 0x0, 0x80, 0xc7, 0x3c, 0x88, 0x81],
};

pub const LOADED_IMAGE_PROTOCOL: Guid = Guid {
    data1: 0x5B1B31A1,
    data2: 0x9562,
    data3: 0x11d2,
    data4: [0x8E,0x3F,0x00,0xA0,0xC9,0x69,0x72,0x3B],
};

pub const DEVICE_PATH_GUID: Guid = Guid {
    data1: 0x09576e91,
    data2: 0x6d3f,
    data3: 0x11d2,
    data4: [0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b],
};

pub const DEVICE_PATH_TO_TEXT_GUID: Guid = Guid {
    data1: 0x8b843e20,
    data2: 0x8132,
    data3: 0x4852,
    data4: [0x90,0xcc,0x55,0x1a,0x4e,0x4a,0x7f,0x1c],
};

pub const LOAD_FILE_GUID: Guid = Guid {
    data1: 0x56EC3091,
    data2: 0x954C,
    data3: 0x11d2,
    data4: [0x8e,0x3f,0x00,0xa0, 0xc9,0x69,0x72,0x3b],
};

pub const LOAD_FILE2_GUID: Guid = Guid {
    data1: 0x4006c0c1,
    data2: 0xfcb3,
    data3: 0x403e,
    data4: [0x99,0x6d,0x4a,0x6c,0x87,0x24,0xe0,0x6d],
};

pub const SIMPLE_FILE_SYSTEM_GUID: Guid = Guid {
    data1: 0x0964e5b22,
    data2: 0x6459,
    data3: 0x11d2,
    data4: [0x8e,0x39,0x00,0xa0,0xc9,0x69,0x72,0x3b],
};

pub const FILE_GUID: Guid = Guid {
    data1: 0,
    data2: 0,
    data3: 0,
    data4: [0; 8],
};


/// Single row in the vendor configuration table
#[repr(C)]
pub struct ConfigurationTable {
    vendor_guid:    Guid,
    vendor_table:   def::Handle,
}

impl ::bind::EfiParameter for ResetType {
    fn as_usize(&self) -> usize {
        *self as usize
    }
}

pub type FunctionPointer = def::Handle;

pub type Event = def::Handle;


#[repr(C)]
pub struct SystemTable {
    hdr:                            TableHeader,

    pub firmware_vendor:            *const u16,
    pub firmware_revision:          u32,

    console_in_handle:              def::Handle,
    con_in: &'static ::api::protocol::SimpleTextInputProtocol,

    console_out_handle:             def::Handle,
    con_out:&'static ::api::protocol::SimpleTextOutputProtocol,

    standard_error_handle:          def::Handle,
    std_err:&'static ::api::protocol::SimpleTextOutputProtocol,

    pub runtime_services:           &'static RuntimeServices,
    pub boot_services:              &'static BootServices,

    number_of_table_entries:        usize,
    configuration_table:            *const ConfigurationTable,

}

impl SystemTable {
    pub fn configuration_table<'a>(&'a self) -> &'a [ConfigurationTable] {
        unsafe {
            slice::from_raw_parts(
                self.configuration_table,
                self.number_of_table_entries)
        }
    }
    pub fn get_vendor_table(&self, guid:&Guid) -> Option<def::Handle> {
        for table in self.configuration_table() {
            if table.vendor_guid == *guid {
                return Some(table.vendor_table);
            }
        }
        None
    }
}

pub struct EfiBuffer {
    buffer: ::core::nonzero::NonZero<*mut u8>,
    size: usize,
}

static mut BOOT_SERVICES: Option<&'static BootServices> = None;

impl EfiBuffer {
    pub unsafe fn init_dealloc(boot_services: &BootServices) {
        BOOT_SERVICES = Some(::core::mem::transmute(boot_services));
    }

    pub unsafe fn new(pointer: *mut u8, size: usize) -> EfiBuffer {
        EfiBuffer {
            buffer: ::core::nonzero::NonZero::new(pointer).unwrap(),
            size: size,
        }
    }

    pub fn as_ptr(&self) -> *const u8 {
        *self.get_pointer()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.get_mut_pointer()
    }

    pub fn get_pointer<'a>(&'a self) -> &'a *const u8 {
        unsafe {
            ::core::mem::transmute(self.buffer.get())
        }
    }

    pub fn get_mut_pointer(& mut self) -> *mut u8 {
        self.buffer.get()
    }

    pub fn into_raw_parts(self) -> (*mut u8, usize) {
        let result = (self.buffer.get(), self.size);
        ::core::mem::forget(self);
        result
    }
}

impl Drop for EfiBuffer {
    fn drop(&mut self) {
        unsafe {
            BOOT_SERVICES.unwrap().free_pool(&self).unwrap();
        }
    }
}
