

use ::mem::{Page, PhysicalAddress};

pub static mut PAGE_ALLOCATOR: PageAllocator = PageAllocator {
    next_page: 0,
};

pub struct PageAllocator {
    next_page: usize,
}

impl PageAllocator {
    pub fn get_page(&mut self) -> Page {
        let result = Page::new(self.next_page);
        self.next_page += 1;
        result
    }

    pub fn get_multiple_pages(&mut self, num_pages: usize) -> Page {
        let result = Page::new(self.next_page);
        self.next_page += num_pages;
        result
    }
}
