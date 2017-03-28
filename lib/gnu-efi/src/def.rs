

pub use err::Status;

/// Handle to be passed to UEFI functions
#[derive(Clone, Copy)]
pub struct Handle {
    pub handle:*const ::mem::c_void,
}

pub use self::memory_descriptor::{MemoryDescriptor, MemoryDescriptors, MemoryType, AllocateType};

mod memory_descriptor {
    /// Struct that lists the memory regions in use by 
    /// different parts of UEFI and bios code
    #[repr(C)]
    #[derive(Debug)]
    pub struct MemoryDescriptor {
        pub region_type:        MemoryType,
        pub physical_start:     ::mem::PhysicalAddress,
        pub virtual_start:      ::mem::VirtualAddress,
        pub number_of_pages:    u64,
        pub attribute:          u64,
    }

    pub struct MemoryDescriptors {
        start: *const MemoryDescriptor,
        number: usize,
        size: usize,
    }

    impl MemoryDescriptors {
        pub fn new(start: *const MemoryDescriptor, number: usize, size: usize) -> MemoryDescriptors {
            MemoryDescriptors {
                start: start,
                number: number,
                size: size,
            }
        }
        pub fn len(&self) -> usize {
            self.number
        }

        pub fn get(&self, index: usize) -> Option<&MemoryDescriptor> {
            if index < self.number {
                unsafe { Some(self.get_unchecked(index)) }
            } else {
                None
            }
        }

        pub unsafe fn get_unchecked(&self, index: usize) -> &MemoryDescriptor {
            let pointer = self.start as *const u8;
            ::core::mem::transmute(pointer.offset((self.size * index) as isize))
        }
    }

    impl<'a> IntoIterator for &'a MemoryDescriptors {
        type Item = &'a MemoryDescriptor;
        type IntoIter = Iter<'a>;
        fn into_iter(self) -> Self::IntoIter {
            Iter {
                current: 0,
                descriptors: self,
            }
        }
    }

    pub struct Iter<'a> {
        current: usize,
        descriptors: &'a MemoryDescriptors,
    }

    impl<'a> Iterator for Iter<'a> {
        type Item = &'a MemoryDescriptor;
        fn next(&mut self) -> Option<Self::Item> {
            if self.current < self.descriptors.number {
                let result = unsafe {
                    self.descriptors.get_unchecked(self.current)
                };
                self.current += 1;
                Some(result)
            } else {
                None
            }
        }
    }

    #[repr(u64)]
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub enum MemoryType {
        ReservedMemoryType,
        LoaderCode,
        LoaderData,
        BootServicesCode,
        BootServicesData,
        RuntimeServicesCode,
        RuntimeServicesData,
        ConventionalMemory,
        UnusableMemory,
        ACPIReclaimMemory,
        ACPIMemoryNVS,
        MemoryMappedIO,
        MemoryMappedIOPortSpace,
        PalCode,
        PersistentMemory,
        MaxMemoryType,
    }

    #[repr(u64)]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum AllocateType {
        AllocateAnyPages,
        AllocateMaxAddress,
        AllocateAddress,
        MaxAllocateType,
    }
}

