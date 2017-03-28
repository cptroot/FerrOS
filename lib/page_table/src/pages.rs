
use core::ops::Add;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PageOffset {
    page_offset: isize,
}

impl PageOffset {
    pub fn new(page_offset: isize) -> Self {
        PageOffset {
            page_offset: page_offset,
        }
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
    fn step(&self, by: &Self) -> Option<Self> {
        isize::checked_add(self.page_offset, by.page_offset).map(
            |page_offset| {
                PageOffset {
                    page_offset: page_offset,
                }
            })
    }

    fn steps_between(start: &Self, end: &Self, by: &Self) -> Option<usize> {
        if by.page_offset < 0 {
            if start.page_offset < end.page_offset {
                None
            } else {
                Some(((start.page_offset - end.page_offset) as usize + (-by.page_offset) as usize - 1) / (-by.page_offset) as usize)
            }
        } else {
            if start.page_offset > end.page_offset {
                None
            } else {
                Some(((end.page_offset - start.page_offset) as usize + by.page_offset as usize - 1) / by.page_offset as usize)
            }
        }
    }

    fn steps_between_by_one(start: &Self, end: &Self) -> Option<usize> {
        if start.page_offset > end.page_offset {
            None
        } else {
            Some((start.page_offset - end.page_offset) as usize)
        }
    }

    fn is_negative(&self) -> bool {
        self.page_offset < 0
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
}

impl From<isize> for PageOffset {
    fn from(value: isize) -> Self {
        PageOffset::new(value)
    }
}

impl From<PageOffset> for isize {
    fn from(value: PageOffset) -> Self {
        value.page_offset
    }
}

impl From<PageOffset> for super::AddressOffset {
    fn from(value: PageOffset) -> super::AddressOffset {
        Self::new(value.page_offset * super::PageSize::FourKb as isize)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PageAddress {
    page_address: usize,
}

impl PageAddress {
    pub const fn new(page_address: usize) -> Self {
        PageAddress {
            page_address: page_address,
        }
    }
}

impl From<usize> for PageAddress {
    fn from(value: usize) -> Self {
        PageAddress::new(value)
    }
}

impl From<PageAddress> for usize {
    fn from(value: PageAddress) -> Self {
        value.page_address
    }
}

impl From<PageAddress> for super::PhysicalAddress {
    fn from(value: PageAddress) -> super::PhysicalAddress {
        Self::new(value.page_address * super::PageSize::FourKb as usize)
    }
}

impl Add<PageOffset> for PageAddress {
    type Output = PageAddress;
    fn add(self, rhs: PageOffset) -> Self::Output {
        PageAddress {
            page_address: 
                if rhs.page_offset < 0 {
                    self.page_address - (-rhs.page_offset) as usize
                } else {
                    self.page_address + rhs.page_offset as usize
                },
        }
    }
}
