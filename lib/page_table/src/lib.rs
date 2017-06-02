#![no_std]
#![feature(unique)]

extern crate x86;
extern crate rlibc;

extern crate mem;
extern crate frame_allocator as falloc;


use core::ptr::Unique;

#[derive(Clone, Copy)]
pub enum PageSize {
    FourKb = 0x1000,
    TwoMb = 0x2_00000,
    OneGb = 0x400_00000,
}

pub struct PageTable {
    pml4: Unique<level4::PageMap>,
}

impl PageTable {
    pub unsafe fn new(frame: ::mem::Frame) -> PageTable {
        let mut physical_address: ::mem::PhysicalAddress = frame.into();
        let mut result = PageTable {
            pml4: Unique::new(physical_address.as_mut_ptr() as *mut level4::PageMap),
        };
        #[cfg(feature = "loader")]
        {
            *result.pml4.as_mut() = level4::PageMap::new();
        }
        #[cfg(feature = "kernel")]
        {
        }
        result
    }

    pub fn insert_page(&mut self, frame: ::mem::Frame, page: ::mem::Page, page_size: PageSize) {
        unsafe {
            (*self.pml4.as_mut()).insert_page(frame, page, page_size);
        }
    }

    pub fn load(&self) {
        unsafe {
            let cr3 = ::x86::shared::control_regs::cr3();
            ::x86::shared::control_regs::cr3_write(self.pml4.as_ptr() as usize);
        }
    }

    pub fn physical_address(&self) -> u32 {
        (self.pml4.as_ptr() as usize) as u32
    }
}

mod level4 {
    use ::mem::{Frame, Page};
    use super::{level3, PageEntryBuilder, ReadWrite, PageSize};

    pub struct PageMap {
        table: [GenericPageEntry; 0x200],
    }

    impl PageMap {
        pub fn new() -> Self {
            PageMap {
                table: [GenericPageEntry::empty(); 0x200],
            }
        }

        pub fn insert_page(&mut self, frame: Frame, page: Page, page_size: PageSize) {
            let index = {
                let virtual_address: ::mem::VirtualAddress = page.into();
                let virtual_address: usize = virtual_address.into();
                (virtual_address >> 39) & 0x1FF
            };
            if let PageEntryType::NotPresent(_) = self.table[index].as_enum() {
                // Insert new page map level 3
                let frame = unsafe { ::falloc::FRAME_ALLOCATOR.get_frame() };
                let physical_address: ::mem::PhysicalAddress = frame.into();
                let ptr = physical_address.as_ptr() as *mut level3::PageMap;
                level3::PageMap::new_in_place(ptr);

                self.table[index] = PageTableEntry::new(frame).into();
            }

            self.table[index].get_map().insert_page(frame, page, page_size);
        }
    }

    #[derive(Clone, Copy)]
    struct GenericPageEntry {
        entry: u64,
    }

    impl GenericPageEntry {
        pub fn empty() -> Self {
            GenericPageEntry {
                entry: 0
            }
        }

        pub fn as_enum(self) -> PageEntryType {
            unsafe {
                use ::core::mem::transmute;
                use self::PageEntryType::*;
                match self.get_present() {
                    false => NotPresent(transmute(self)),
                    true => PageTableEntry(transmute(self)),
                }
            }
        }

        pub fn get_present(&self) -> bool {
            (self.entry & 0x1) == 0x1
        }

        pub fn get_map(&mut self) -> &mut level3::PageMap {
            let mut physical_address = ::mem::PhysicalAddress::new(self.entry as usize & (0xFFFFFFFFFF << 12));
            let ptr = physical_address.as_mut_ptr() as *mut level3::PageMap;
            unsafe { ::core::mem::transmute(ptr) }
        }
    }

    enum PageEntryType {
        NotPresent(NotPresent),
        PageTableEntry(PageTableEntry),
    }

    struct NotPresent {
        entry: u64,
    }

    impl From<NotPresent> for GenericPageEntry {
        fn from(value: NotPresent) -> Self {
            GenericPageEntry {
                entry: value.entry
            }
        }
    }

    struct PageTableEntry {
        entry: u64,
    }

    impl PageTableEntry {
        pub fn new(frame: Frame) -> Self {
            let mut entry: u64 = 
                (PageEntryBuilder {
                    read_write: ReadWrite::Writeable,
                    ..Default::default()
                }).to_entry();
            entry |= 0x1 << 0;
            let frame_number: usize = frame.into();
            entry |= (frame_number as u64 & 0xFFFFF_FFFFF) << 12;
            PageTableEntry {
                entry: entry,
            }
        }
    }

    impl From<PageTableEntry> for GenericPageEntry {
        fn from(value: PageTableEntry) -> Self {
            GenericPageEntry {
                entry: value.entry
            }
        }
    }
}

mod level3 {
    use ::mem::{Frame, Page};
    use super::{level2, PageEntryBuilder, ReadWrite, PageSize};

    pub struct PageMap {
        table: [GenericPageEntry; 0x200],
    }

    impl PageMap {
        pub fn new_in_place(page_map: *mut PageMap) {
            unsafe {
                ::rlibc::memset(page_map as *mut u8, 0, 0x1000);
            }
        }

        pub fn insert_page(&mut self, frame: Frame, page: Page, page_size: PageSize) {
            let index = {
                let virtual_address: ::mem::VirtualAddress = page.into();
                let virtual_address: usize = virtual_address.into();
                (virtual_address >> 30) & 0x1FF
            };
            match (page_size, self.table[index].as_enum()) {
                (PageSize::OneGb, PageEntryType::NotPresent(_)) => {
                    self.table[index] = PageEntry::new(frame).into();
                },
                (PageSize::OneGb, PageEntryType::PageTableEntry(_)) => {
                    panic!("Can't insert a 1gb page. There's already a page table here");
                },
                (PageSize::OneGb, PageEntryType::PageEntry(_)) => {
                    panic!("This page is already mapped {:?} : {:?}", page, frame);
                },
                (_, PageEntryType::NotPresent(_)) => {
                    // Insert a new page table entry
                    let entry = {
                        let frame = unsafe { ::falloc::FRAME_ALLOCATOR.get_frame() };
                        let physical_address: ::mem::PhysicalAddress = frame.into();
                        let entry = PageTableEntry::new(frame);
                        self.table[index] = entry.into();

                        let ptr = physical_address.as_ptr() as *mut level2::PageMap;
                        level2::PageMap::new_in_place(ptr);
                        entry
                    };
                        
                    entry.get_map().insert_page(frame, page, page_size);
                },
                (_, PageEntryType::PageTableEntry(entry)) => {
                    entry.get_map().insert_page(frame, page, page_size);
                },
                (_, PageEntryType::PageEntry(_)) => {
                    panic!("Can't insert a smaller page. There's already a 1gb page here.");
                },
            };
        }
    }

    impl Default for PageMap {
        fn default() -> Self {
            PageMap {
                table: [Default::default(); 0x200],
            }
        }
    }

    #[derive(Clone, Copy)]
    struct GenericPageEntry {
        entry: u64,
    }

    impl GenericPageEntry {
        pub fn as_enum(self) -> PageEntryType {
            let present = self.present();
            let page_size = self.page_size();
            unsafe {
                use ::core::mem::transmute;
                match (present, page_size) {
                    (false, _) =>
                        PageEntryType::NotPresent(transmute(self)),
                    (true, false) =>
                        PageEntryType::PageTableEntry(transmute(self)),
                    (true, true) =>
                        PageEntryType::PageEntry(transmute(self)),
                }
            }
        }

        pub fn present(&self) -> bool {
            (self.entry & 0x1) == 0x1
        }

        pub fn page_size(&self) -> bool {
            ((self.entry >> 7) & 0x1) == 0x1
        }
    }

    impl Default for GenericPageEntry {
        fn default() -> Self {
            GenericPageEntry {
                entry: 0,
            }
        }
    }

    enum PageEntryType {
        NotPresent(NotPresent),
        PageTableEntry(PageTableEntry),
        PageEntry(PageEntry),
    }

    #[derive(Clone, Copy)]
    struct NotPresent {
        entry: u64,
    }

    impl From<NotPresent> for GenericPageEntry {
        fn from(value: NotPresent) -> Self {
            GenericPageEntry {
                entry: value.entry
            }
        }
    }

    #[derive(Clone, Copy)]
    struct PageTableEntry {
        entry: u64,
    }

    impl PageTableEntry {
        pub fn new(frame: Frame) -> Self {
            let mut entry: u64 = 
                (PageEntryBuilder {
                    read_write: ReadWrite::Writeable,
                    ..Default::default()
                }).to_entry();
            entry |= 0x1 << 0;
            let frame_number: usize = frame.into();
            entry |= (frame_number as u64 & 0xFFFFF_FFFFF) << 12;
            PageTableEntry {
                entry: entry,
            }
        }

        pub fn get_map(&self) -> &mut level2::PageMap {
            let mut physical_address = ::mem::PhysicalAddress::new(self.entry as usize & (0xFFFFFFFFFF << 12));
            let ptr = physical_address.as_mut_ptr() as *mut level2::PageMap;
            unsafe { ::core::mem::transmute(ptr) }
        }
    }

    impl From<PageTableEntry> for GenericPageEntry {
        fn from(value: PageTableEntry) -> Self {
            GenericPageEntry {
                entry: value.entry
            }
        }
    }

    #[derive(Clone, Copy)]
    struct PageEntry {
        entry: u64,
    }

    impl PageEntry {
        pub fn new(frame: Frame) -> Self {
            let mut entry: u64 = 
                (PageEntryBuilder {
                    read_write: ReadWrite::Writeable,
                    ..Default::default()
                }).to_entry();
            entry |= 0x1 << 0;
            entry |= 0x1 << 7;
            let frame_number: usize = frame.into();
            entry |= (frame_number as u64 & 0x3FFFFF << 18) << 12;
            PageEntry {
                entry: entry,
            }
        }
    }

    impl From<PageEntry> for GenericPageEntry {
        fn from(value: PageEntry) -> Self {
            GenericPageEntry {
                entry: value.entry
            }
        }
    }
}

mod level2 {
    use ::mem::{Frame, Page};
    use super::{level1, PageEntryBuilder, ReadWrite, PageSize};

    pub struct PageMap {
        table: [GenericPageEntry; 0x200],
    }

    impl Default for PageMap {
        fn default() -> Self {
            PageMap {
                table: [Default::default(); 0x200],
            }
        }
    }

    impl PageMap {
        pub fn new_in_place(page_map: *mut PageMap) {
            unsafe {
                ::rlibc::memset(page_map as *mut u8, 0, 0x1000);
            }
        }

        pub fn insert_page(&mut self, frame: Frame, page: Page, page_size: PageSize) {
            let index = {
                let virtual_address: ::mem::VirtualAddress = page.into();
                let virtual_address: usize = virtual_address.into();
                (virtual_address >> 21) & 0x1FF
            };
            match (page_size, self.table[index].as_enum()) {
                (PageSize::TwoMb, PageEntryType::NotPresent(_)) => {
                    self.table[index] = PageEntry::new(frame).into();
                },
                (PageSize::TwoMb, PageEntryType::PageTableEntry(_)) => {
                    panic!("Can't insert a 2mb page. There's already a page table here");
                },
                (PageSize::TwoMb, PageEntryType::PageEntry(_)) => {
                    panic!("This page is already mapped {:?} : {:?}", page, frame);
                },
                (PageSize::FourKb, PageEntryType::NotPresent(_)) => {
                    // Insert a new page table entry
                    let entry = {
                        let frame = unsafe { ::falloc::FRAME_ALLOCATOR.get_frame() };
                        let physical_address: ::mem::PhysicalAddress = frame.into();
                        let entry = PageTableEntry::new(frame);
                        self.table[index] = entry.into();

                        let ptr = physical_address.as_ptr() as *mut level1::PageMap;
                        level1::PageMap::new_in_place(ptr);
                        entry
                    };

                    entry.get_map().insert_page(frame, page, page_size);
                },
                (PageSize::FourKb, PageEntryType::PageTableEntry(entry)) => {
                    entry.get_map().insert_page(frame, page, page_size);
                },
                (PageSize::FourKb, PageEntryType::PageEntry(_)) => {
                    panic!("Can't insert a smaller page. There's already a 1gb page here.");
                },
                (PageSize::OneGb, _) => unreachable!(),
            };
        }
    }

    #[derive(Clone, Copy)]
    struct GenericPageEntry {
        entry: u64,
    }

    impl Default for GenericPageEntry {
        fn default() -> Self {
            GenericPageEntry {
                entry: 0,
            }
        }
    }

    impl GenericPageEntry {
        pub fn as_enum(self) -> PageEntryType {
            let present = self.present();
            let page_size = self.page_size();

            unsafe {
                use ::core::mem::transmute;
                use self::PageEntryType::*;
                match (present, page_size) {
                    (false, _) => NotPresent(transmute(self)),
                    (true, false) => PageTableEntry(transmute(self)),
                    (true, true) => PageEntry(transmute(self)),
                }
            }
        }

        pub fn present(&self) -> bool {
            (self.entry & 0x1) == 0x1
        }

        pub fn page_size(&self) -> bool {
            ((self.entry >> 7) & 0x1) == 0x1
        }
    }

    enum PageEntryType {
        NotPresent(NotPresent),
        PageTableEntry(PageTableEntry),
        PageEntry(PageEntry),
    }

    #[derive(Clone, Copy)]
    struct NotPresent {
        entry: u64,
    }

    #[derive(Clone, Copy)]
    struct PageTableEntry {
        entry: u64,
    }

    impl PageTableEntry {
        pub fn new(frame: Frame) -> Self {
            let mut entry: u64 = 
                (PageEntryBuilder {
                    read_write: ReadWrite::Writeable,
                    ..Default::default()
                }).to_entry();
            entry |= 0x1 << 0;
            let frame_number: usize = frame.into();
            entry |= (frame_number as u64 & 0xFFFFF_FFFFF) << 12;
            PageTableEntry {
                entry: entry,
            }
        }

        pub fn get_map(&self) -> &mut level1::PageMap {
            let mut physical_address = ::mem::PhysicalAddress::new(self.entry as usize & (0xFFFFFFFFFF << 12));
            let ptr = physical_address.as_mut_ptr() as *mut level1::PageMap;
            unsafe { ::core::mem::transmute(ptr) }
        }
    }

    impl From<PageTableEntry> for GenericPageEntry {
        fn from(value: PageTableEntry) -> Self {
            GenericPageEntry {
                entry: value.entry
            }
        }
    }

    #[derive(Clone, Copy)]
    struct PageEntry {
        entry: u64,
    }
    
    impl PageEntry {
        pub fn new(frame: Frame) -> Self {
            let mut entry: u64 = 
                (PageEntryBuilder {
                    read_write: ReadWrite::Writeable,
                    ..Default::default()
                }).to_entry();
            entry |= 0x1 << 0;
            entry |= 0x1 << 7;
            let frame_number: usize = frame.into();
            entry |= (frame_number as u64 & 0x7FFFFFFF << 9) << 12;
            PageEntry {
                entry: entry,
            }
        }
    }

    impl From<PageEntry> for GenericPageEntry {
        fn from(value: PageEntry) -> Self {
            GenericPageEntry {
                entry: value.entry
            }
        }
    }
}

mod level1 {
    use ::mem::{Frame, Page};
    use super::{PageEntryBuilder, ReadWrite, PageSize};

    pub struct PageMap {
        table: [GenericPageEntry; 0x200],
    }

    impl Default for PageMap {
        fn default() -> Self {
            PageMap {
                table: [Default::default(); 0x200],
            }
        }
    }

    impl PageMap {
        pub fn new_in_place(page_map: *mut PageMap) {
            unsafe {
                ::rlibc::memset(page_map as *mut u8, 0, 0x1000);
            }
        }

        pub fn insert_page(&mut self, frame: Frame, page: Page, page_size: PageSize) {
            let index = {
                let virtual_address: ::mem::VirtualAddress = page.into();
                let virtual_address: usize = virtual_address.into();
                (virtual_address >> 12) & 0x1FF
            };
            match (page_size, self.table[index].as_enum()) {
                (PageSize::FourKb, PageEntryType::NotPresent(_)) => {
                    self.table[index] = PageEntry::new(frame).into();
                },
                (PageSize::FourKb, PageEntryType::PageEntry(_)) => {
                    panic!("This page is already mapped {:?} : {:?}", page, frame);
                },
                (_, _) => unreachable!(),
            };
        }
    }

    #[derive(Clone, Copy)]
    struct GenericPageEntry {
        entry: u64,
    }

    impl Default for GenericPageEntry {
        fn default() -> Self {
            GenericPageEntry {
                entry: 0,
            }
        }
    }

    impl GenericPageEntry {
        pub fn as_enum(self) -> PageEntryType {
            unsafe {
                use ::core::mem::transmute;
                use self::PageEntryType::*;
                match self.present() {
                    false => NotPresent(transmute(self)),
                    true => PageEntry(transmute(self)),
                }
            }
        }

        pub fn present(&self) -> bool {
            (self.entry & 0x1) == 0x1
        }
    }

    enum PageEntryType {
        NotPresent(NotPresent),
        PageEntry(PageEntry),
    }

    struct NotPresent {
        entry: u64,
    }

    struct PageEntry {
        entry: u64,
    }
    
    impl PageEntry {
        pub fn new(frame: Frame) -> Self {
            let mut entry: u64 = 
                (PageEntryBuilder {
                    read_write: ReadWrite::Writeable,
                    ..Default::default()
                }).to_entry();
            entry |= 0x1 << 0;
            let frame_number: usize = frame.into();
            entry |= (frame_number as u64 & 0xFFFFF_FFFFF) << 12;
            PageEntry {
                entry: entry,
            }
        }
    }

    impl From<PageEntry> for GenericPageEntry {
        fn from(value: PageEntry) -> Self {
            GenericPageEntry {
                entry: value.entry
            }
        }
    }
}

struct PageEntryBuilder {
    read_write: ReadWrite,
    user_supervisor: UserSupervisor,
    page_level_write_through: bool,
    page_level_cache_disable: bool,
    accessed: bool,
}

impl Default for PageEntryBuilder {
    fn default() -> Self {
        PageEntryBuilder {
            read_write: ReadWrite::ReadOnly,
            user_supervisor: UserSupervisor::Supervisor,
            page_level_write_through: false,
            page_level_cache_disable: false,
            accessed: false,
        }
    }
}

impl PageEntryBuilder {
    pub fn to_entry(self) -> u64 {
        let mut result: u64 = 0;
        result |= (self.read_write as u64) << 1;
        result |= (self.user_supervisor as u64) << 2;
        result |= (self.page_level_write_through as u64) << 3;
        result |= (self.page_level_cache_disable as u64) << 4;
        result |= (self.accessed as u64) << 5;
        result
    }
}

enum ReadWrite {
    ReadOnly = 0,
    Writeable = 1,
}

enum UserSupervisor {
    Supervisor = 0,
    User = 1,
}


