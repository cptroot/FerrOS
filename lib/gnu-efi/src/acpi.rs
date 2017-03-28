
use core::mem;
use core::slice;

use api;

const RSDP_SIGNATURE: &'static [u8; 8] = b"RSD PTR ";

unsafe fn verify_checksum(ptr: *const u8, length: usize) -> bool {
    let mut total: u8 = 0;
    for byte in slice::from_raw_parts(ptr, length) {
        total = total.wrapping_add(*byte);
    }
    total == 0
}

#[repr(packed)]
pub struct RootSystemDescriptorPointer {
    signature:          [u8; 8],
    _checksum:          u8,
    _oem_id:            [u8; 6],
    revision:           u8,
    _rsdt_address:      u32,
    length:             u32,
    pub xsdt:           &'static ExtendedSystemDescriptorTable,
    _extended_checksum: u8,
    _reserved:          [u8; 3],
}

impl RootSystemDescriptorPointer {
    fn verify_signature(&self) -> bool {
        self.signature == *RSDP_SIGNATURE
    }
    fn verify_checksum(&self) -> bool {
        unsafe {
            verify_checksum((self as *const Self) as *const u8, 20)
        }
    }
    fn verify_extended_checksum(&self) -> bool {
        unsafe {
            verify_checksum((&self.length as *const u32) as *const u8, 33 - 20)
        }
    }
    fn verify_length(&self) -> bool {
        self.length == 36 || self.length == 33
    }
    fn verify_revision(&self) -> bool {
        self.revision >= 2
    }
    pub fn verify(&self) -> bool {
        self.verify_signature() &&
            self.verify_checksum() &&
            self.verify_extended_checksum() &&
            self.verify_length() &&
            self.verify_revision()
    }
}

#[repr(packed)]
pub struct SystemDescriptionTableHeader {
    signature:          [u8; 4],
    length:             u32,
    _revision:           u8,
    _checksum:          u8,
    _oem_id:            [u8; 6],
    _oem_table_id:      [u8; 8],
    _oem_revision:      u32,
    _creator_id:        u32,
    _creator_revision:  u32,
}

const XSDT_SIGNATURE: &'static [u8; 4] = b"XSDT";

#[repr(packed)]
pub struct ExtendedSystemDescriptorTable {
    header: SystemDescriptionTableHeader,
    entry:  &'static SystemDescriptionTableHeader,
}

impl ExtendedSystemDescriptorTable {
    pub fn verify(&self) -> bool {
        self.verify_signature() &&
            self.verify_checksum()
    }
    fn verify_signature(&self) -> bool {
        self.header.signature == *XSDT_SIGNATURE
    }
    fn verify_checksum(&self) -> bool {
        unsafe {
            verify_checksum(mem::transmute(self), self.header.length as usize)
        }
    }

    pub fn get_tables<'a>(&'a self) -> &'a [&'static SystemDescriptionTableHeader] {
        unsafe {
            let mut bytes = self.header.length as usize;
            bytes = bytes - mem::size_of::<SystemDescriptionTableHeader>();
            bytes = bytes / 8;
            slice::from_raw_parts(&self.entry, bytes)
        }
    }

    pub fn find_sdt_by_signature(&self, signature:&[u8; 4]) -> Option<&'static SystemDescriptionTableHeader> {
        for table in self.get_tables() {
            if table.signature == *signature {
                return Some(table);
            }
        }
        None
    }

    pub fn find_madt(&self) -> Option<&'static MultipleApicDescriptionTable> {
        if let Some(header) = self.find_sdt_by_signature(b"APIC") {
            unsafe {
                let header_ptr: *const SystemDescriptionTableHeader = header;
                Some(mem::transmute(header_ptr))
            }
        } else {
            None
        }
    }
}

#[repr(packed)]
pub struct InterruptControllerHeader {
    pub structure_type: u8,
    length: u8,
}

impl InterruptControllerHeader {
    pub fn to_interrupt_source_override<'a>(&'a self) -> &'a InterruptSourceOverride {
        assert!(self.structure_type == 2);
        unsafe {
            mem::transmute(self)
        }
    }
}

pub struct InterruptControllers {
    ptr: *const ::mem::c_void,
    end_ptr: *const ::mem::c_void,
}

impl Iterator for InterruptControllers {
    type Item = &'static InterruptControllerHeader;
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.end_ptr {
            None
        } else {
            unsafe {
                let result: &'static InterruptControllerHeader = mem::transmute(self.ptr);
                self.ptr = self.ptr.offset(result.length as isize);
                Some(result)
            }
        }
    }
}

pub struct MultipleApicDescriptionTable {
    header: SystemDescriptionTableHeader,
    _local_interrupt_controller_address: u32,
    _flags: u32,
    interrupt_controller_structure: ::mem::c_void,
}

impl MultipleApicDescriptionTable {
    pub fn controllers(&self) -> InterruptControllers {
        unsafe {
            InterruptControllers {
                ptr: &self.interrupt_controller_structure as *const ::mem::c_void,
                end_ptr: ((self as *const Self) as *const ::mem::c_void).offset(self.header.length as isize),
            }
        }
    }
}

#[repr(C)]
pub struct InterruptSourceOverride {
    structure_type: u8,
    length: u8,
    bus: u8,
    pub source: u8,
    pub global_system_interrupt: u32,
    flags: u16,
}

pub fn get_rsdp(system_table: &api::SystemTable) ->
        &'static RootSystemDescriptorPointer {
    let rsdp: &'static RootSystemDescriptorPointer =
        unsafe {
            mem::transmute(
                system_table.get_vendor_table(
                    &api::types::ACPI_20_TABLE_GUID).expect(
                        "No ACPI Table found."))
        };
    rsdp
}
