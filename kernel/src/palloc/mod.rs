use ::core::ptr::Unique;

use ::core::ops::{Index, IndexMut};

use ::mem::{Page, PageOffset, FrameOffset, VirtualAddress};
use ::spin::Mutex;

pub static PAGE_ALLOCATOR: Mutex<PageAllocator> = Mutex::new(PageAllocator {
    next_page: 0x81_0000_0,
});

pub struct PageAllocator {
    next_page: usize,
}

impl PageAllocator {
    pub fn get_page(&mut self) -> OwnedPage {
        let frame = ::FRAME_ALLOCATOR.lock().get_frame();
        let page = Page::new(self.next_page);

        ::PAGE_TABLE.lock().insert_page::<::FramePlace>(frame, page, ::page_table::PageSize::FourKb);

        self.next_page += 1;
        OwnedPage {
            page,
        }
    }

    pub fn get_multiple_pages(&mut self, num_pages: usize) -> OwnedPages {
        let frame = ::FRAME_ALLOCATOR.lock().get_multiple_frames(num_pages);
        let page = Page::new(self.next_page);

        let mut page_table = ::PAGE_TABLE.lock();
        for offset in 0..num_pages {
            let page = page + PageOffset::new(offset as isize);
            let frame = frame + FrameOffset::new(offset as isize);

            page_table.insert_page::<::FramePlace>(frame, page, ::page_table::PageSize::FourKb);
        }

        self.next_page += num_pages;
        OwnedPages {
           page,
           len: num_pages,
        }
    }
}

pub struct OwnedPage {
    page: Page,
}

pub struct OwnedPages {
    page: Page,
    len: usize,
}

impl OwnedPages {
    pub fn into_unique<T>(self) -> (Unique<T>, usize) {
        unsafe {
            let mut virtual_address: VirtualAddress = self.page.into();
            (
                Unique::new_unchecked(virtual_address.as_mut_ptr() as *mut T),
                self.len
            )
        }
    }
}

const PAGE_SIZE: usize = 1 << 12;

pub struct PallocArray<T> {
    ptr: Unique<T>,
    len: usize,
}

pub struct IterMut<'a, T> where T: 'a {
    ptr: *mut u8,
    end: *mut u8,
    phantom_data: ::core::marker::PhantomData<&'a mut T>,
}

impl<T> PallocArray<T> {
    pub fn new(pages: OwnedPages) -> PallocArray<T> {
        let (ptr, len) = pages.into_unique();
        PallocArray {
            ptr,
            len,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        unsafe {
            IterMut {
                ptr: self.ptr.as_ptr() as *mut u8,
                end: (self.ptr.as_ptr() as *mut u8).offset((PAGE_SIZE * self.len) as isize),
                phantom_data: ::core::marker::PhantomData,
            }
        }
    }
}

impl<T> Index<usize> for PallocArray<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe {
            let ptr = self.ptr.as_ptr() as *mut u8;
            let ptr = ptr.offset((PAGE_SIZE * index) as isize) as *mut T;
            &*ptr
        }
    }
}

impl<T> IndexMut<usize> for PallocArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe {
            let ptr = self.ptr.as_ptr() as *mut u8;
            let ptr = ptr.offset((PAGE_SIZE * index) as isize) as *mut T;
            &mut *ptr
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.end {
            None
        } else {
            unsafe {
                let result = Some(&mut *(self.ptr as *mut T));
                self.ptr = (self.ptr).offset(PAGE_SIZE as isize);
                result
            }
        }
    }
}

pub fn get_contiguous_array<T>(len: usize, new: fn() -> T) -> PallocArray<T> {
    let pages = PAGE_ALLOCATOR.lock().get_multiple_pages(len);
    let mut result = PallocArray::new(pages);

    for value in result.iter_mut() {
        unsafe {
            let ptr = value as *mut T;
            ::core::ptr::write(ptr, new());
        }
    }

    result
}
