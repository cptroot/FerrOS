
use ::mem::c_void;

pub enum TimerMode {
    OneShot,
    Periodic,
    TSCDecline,
}

pub struct ApicRegisters {
    ptr: *mut u32,
}

impl ApicRegisters {
    pub fn new(ptr: *mut c_void) -> Self {
        ApicRegisters {
            ptr: ptr as *mut u32,
        }
    }

    pub fn page_in(&self, page_table: &mut ::page_table::PageTable) {
        page_table.insert_page(::mem::PhysicalAddress::new(self.ptr as usize).into(), ::mem::VirtualAddress::new(self.ptr as usize).into(), ::page_table::PageSize::FourKb);
    }

    pub unsafe fn eoi(&mut self) {
        *self.ptr.offset(4 * 0xB) = 0;
    }

    pub fn get_lvt_timer_register(&self) -> u32 {
        unsafe {
            // Get offset 0x320 u32
            *self.ptr.offset(4 * 0x32)
        }
    }

    pub fn set_lvt_timer_register(&mut self, mode: TimerMode, masked: bool, vector: u8) {
        assert!(vector > 31);
        let mut result: u32 = 0;
        // Timer mode
        result = result | (((mode   as u8 & 0x03) as u32) << 17);
        // Masked
        result = result | (((masked as u8 & 0x01) as u32) << 16);
        // Vector
        result = result | (((vector as u8 & 0xFF) as u32) <<  0);

        unsafe {
            *self.ptr.offset(4 * 0x32) = result;
        }
    }

    pub fn get_timer_initial_count_register(&self) -> u32 {
        unsafe {
            // Get offset 0x380 u32
            *self.ptr.offset(4 * 0x38)
        }
    }

    pub fn set_timer_initial_count_register(&self, count:u32) {
        unsafe {
            *self.ptr.offset(4 * 0x38) = count;
        }
    }
}


