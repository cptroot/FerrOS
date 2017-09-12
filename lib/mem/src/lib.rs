#![no_std]
#![feature(step_trait)]

use core::ops::{Add};

// Use repr(u8) as LLVM expects `void*` to be the same as `i8*` to help enable
// more optimization opportunities around it recognizing things like
// malloc/free.
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum c_void {
    // Two dummy variants so the #[repr] attribute can be used.
    #[doc(hidden)]
    __variant1,
    #[doc(hidden)]
    __variant2,
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Page {
    page: usize,
}

impl Page {
    pub fn new(page: usize) -> Page {
        Page {
            page,
        }
    }
}

impl From<VirtualAddress> for Page {
    fn from(value: VirtualAddress) -> Self {
        Self::new(value.address >> 12)
    }
}

impl Add<PageOffset> for Page {
    type Output = Self;
    fn add(self, rhs: PageOffset) -> Self::Output {
        Page {
            page:
                if rhs.page_offset < 0 {
                    self.page - (-rhs.page_offset) as usize
                } else {
                    self.page + rhs.page_offset as usize
                },
        }
    }
}

impl core::fmt::Debug for Page {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Page {{ page: {:x} }}", self.page)
    }
}

#[derive(Clone, Copy, PartialOrd, PartialEq, Eq, Debug)]
pub struct PageOffset {
    page_offset: isize,
}

impl PageOffset {
    pub fn new(page_offset: isize) -> Self {
        PageOffset {
            page_offset,
        }
    }
}

impl From<VirtualAddressOffset> for PageOffset {
    fn from(value: VirtualAddressOffset) -> Self {
        Self::new((value.address_offset + 0x1000 - 1) / 0x1000)
    }
}

impl From<PageOffset> for isize {
    fn from(value: PageOffset) -> Self {
        value.page_offset
    }
}

impl ::core::ops::Add<PageOffset> for PageOffset {
    type Output = PageOffset;
    fn add(self, rhs: PageOffset) -> Self::Output {
        PageOffset {
            page_offset: self.page_offset + rhs.page_offset,
        }
    }
}

impl<'a> ::core::ops::Add<&'a PageOffset> for &'a PageOffset {
    type Output = PageOffset;
    fn add(self, rhs: &'a PageOffset) -> Self::Output {
        *self + *rhs
    }
}

impl ::core::iter::Step for PageOffset {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        if start.page_offset > end.page_offset {
            None
        } else {
            Some((start.page_offset - end.page_offset) as usize)
        }
    }

    fn replace_one(&mut self) -> Self {
        self.page_offset = 1;
        *self
    }

    fn replace_zero(&mut self) -> Self {
        self.page_offset = 0;
        *self
    }

    fn add_one(&self) -> Self {
        PageOffset {
            page_offset: self.page_offset + 1,
        }
    }

    fn sub_one(&self) -> Self {
        PageOffset {
            page_offset: self.page_offset - 1,
        }
    }

    fn add_usize(&self, n: usize) -> Option<Self> {
        isize::checked_add(self.page_offset, n as isize).map(
            |page_offset| {
                PageOffset {
                    page_offset,
                }
            }
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Frame {
    frame: usize,
}

impl Frame {
    pub fn new(frame: usize) -> Frame {
        Frame {
            frame,
        }
    }
}

impl From<PhysicalAddress> for Frame {
    fn from(value: PhysicalAddress) -> Self {
        Self::new(value.address >> 12)
    }
}

impl From<Frame> for usize {
    fn from(value: Frame) -> Self {
        value.frame
    }
}

impl Add<FrameOffset> for Frame {
    type Output = Self;
    fn add(self, rhs: FrameOffset) -> Self::Output {
        Frame {
            frame:
                if rhs.frame_offset < 0 {
                    self.frame - (-rhs.frame_offset) as usize
                } else {
                    self.frame + rhs.frame_offset as usize
                },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FrameOffset {
    frame_offset: isize,
}

impl FrameOffset {
    pub fn new(frame_offset: isize) -> Self {
        FrameOffset {
            frame_offset,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PhysicalAddress {
    address: usize,
}

impl PhysicalAddress {
    pub fn new(address: usize) -> Self {
        PhysicalAddress {
            address,
        }
    }

    pub fn as_ptr(&self) -> *const c_void {
        self.address as *const c_void
    }

    pub fn as_mut_ptr(&mut self) -> *mut c_void {
        self.address as *mut c_void
    }
}

impl From<Frame> for PhysicalAddress {
    fn from(value: Frame) -> Self {
        PhysicalAddress {
            address: value.frame * 0x1000,
        }
    }
}

impl From<PhysicalAddress> for usize {
    fn from(value: PhysicalAddress) -> Self {
        value.address
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PhysicalAddressOffset {
    address_offset: isize,
}

// Maps one to one with pages
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VirtualAddress {
    address: usize,
}

impl VirtualAddress {
    pub fn new(address: usize) -> Self {
        VirtualAddress {
            address,
        }
    }

    pub fn as_ptr(&self) -> *const usize {
        self.address as *const usize
    }

    pub fn as_mut_ptr(&mut self) -> *mut usize {
        self.address as *mut usize
    }
}

impl From<Page> for VirtualAddress {
    fn from(value: Page) -> Self {
        VirtualAddress {
            address: value.page * 0x1000,
        }
    }
}

impl From<VirtualAddress> for usize {
    fn from(value: VirtualAddress) -> Self {
        value.address
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VirtualAddressOffset {
    address_offset: isize,
}

impl VirtualAddressOffset {
    pub fn new(address_offset: isize) -> Self {
        VirtualAddressOffset {
            address_offset,
        }
    }
}

// Maps one to one with physical addresses
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PageTableAddress {
    address: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PageTableAddressOffset {
    address_offset: isize,
}

// Maps one to one with frames {
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PageTablePage {
    page: usize,
}

// Maps one to one with physical addresses
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UefiAddress {
    address: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UefiAddressOffset {
    address_offset: isize,
}
