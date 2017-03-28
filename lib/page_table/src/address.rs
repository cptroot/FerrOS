
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AddressOffset {
    offset: isize,
}

impl AddressOffset {
    pub fn new(offset: isize) -> Self {
        AddressOffset {
            offset: offset,
        }
    }
}

impl From<AddressOffset> for super::PageOffset {
    fn from(value: AddressOffset) -> Self {
        Self::new((value.offset + super::PageSize::FourKb as isize - 1) / super::PageSize::FourKb as isize)
    }
}

impl From<isize> for AddressOffset {
    fn from(value: isize) -> Self {
        AddressOffset {
            offset: value,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VirtualAddress {
    address: usize,
}

impl ::core::ops::Add<AddressOffset> for VirtualAddress {
    type Output = VirtualAddress;
    fn add(self, rhs: AddressOffset) -> Self::Output {
        VirtualAddress {
            address:
                if rhs.offset < 0 {
                    self.address - (-rhs.offset) as usize
                } else {
                    self.address + rhs.offset as usize
                },
        }
    }
}

impl From<usize> for VirtualAddress {
    fn from(value: usize) -> Self {
        VirtualAddress {
            address: value,
        }
    }
}

impl From<VirtualAddress> for super::PageAddress {
    fn from(value: VirtualAddress) -> Self {
        Self::new((value.address + super::PageSize::FourKb as usize - 1) / super::PageSize::FourKb as usize)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PhysicalAddress {
    address: usize,
}

impl PhysicalAddress {
    pub fn new(address: usize) -> Self {
        PhysicalAddress {
            address: address,
        }
    }

    pub fn as_ptr(&self) -> *const ::gnu_efi::c_void {
        self.address as *const ::gnu_efi::c_void
    }

    pub fn as_mut_ptr(&mut self) -> *mut ::gnu_efi::c_void {
        self.address as *mut ::gnu_efi::c_void
    }
}

impl ::core::ops::Add<AddressOffset> for PhysicalAddress {
    type Output = PhysicalAddress;
    fn add(self, rhs: AddressOffset) -> Self::Output {
        PhysicalAddress {
            address:
                if rhs.offset < 0 {
                    self.address - (-rhs.offset) as usize
                } else {
                    self.address + rhs.offset as usize
                },
        }
    }
}

impl From<usize> for PhysicalAddress {
    fn from(value: usize) -> Self {
        PhysicalAddress {
            address: value,
        }
    }
}

impl From<::gnu_efi::def::PhysicalAddress> for PhysicalAddress {
    fn from(value: ::gnu_efi::def::PhysicalAddress) -> Self {
        PhysicalAddress {
            address: value.address as usize,
        }
    }
}

impl From<PhysicalAddress> for super::PageAddress {
    fn from(value: PhysicalAddress) -> Self {
        Self::new((value.address + super::PageSize::FourKb as usize - 1) / super::PageSize::FourKb as usize)
    }
}

impl From<PhysicalAddress> for usize {
    fn from(value: PhysicalAddress) -> Self {
        value.address
    }
}

