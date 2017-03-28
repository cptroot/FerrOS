#![allow(dead_code)]

use super::INTR_CNT;
pub static mut IDT: [InterruptDescriptor; INTR_CNT] =
    [InterruptDescriptor { data: [0; 4], }; INTR_CNT];

/*
bitfield!{
#[derive(Clone, Copy)]
    pub InterruptDescriptor,
    pub offset_low: 16,
    pub segment_selector: 16,
    pub ist: 3,
    zeros_two: 5,
    pub gate_type: 4,
    zeros_one: 1,
    pub descriptor_privilege_level: 2,
    pub present_flag: 1,
    pub offset_med: 16,
    pub offset_high: 32,
    reserved: 32,
}
*/

#[derive(Clone, Copy)]
pub struct InterruptDescriptor {
    data: [u32; 4],
}

impl InterruptDescriptor {
    pub fn new(data: [u32; 4]) -> InterruptDescriptor {
        InterruptDescriptor {
            data: data,
        }
    }

    pub fn set_offset_low(&mut self, value: u16) {
        self.data[0] = (self.data[0] & 0xFFFF0000) | (value as u32);
    }
    pub fn set_offset_med(&mut self, value: u16) {
        self.data[1] = (self.data[1] & 0x0000FFFF) | ((value as u32) << 16);
    }
    pub fn set_offset_high(&mut self, value: u32) {
        self.data[2] = value;
    }
    pub fn set_present_flag(&mut self, value: bool) {
        if value {
            self.data[1] |= 1 << 15;
        } else {
            self.data[1] &= !(1 << 15);
        }
    }
    pub fn set_gate_type(&mut self, value: super::SystemDescriptorTypes) {
        self.data[1] = (self.data[1] & 0xFFFFF0FF) | ((value as u32) << 8);
    }
    pub fn set_segment_selector(&mut self, value: u16) {
        self.data[0] = (self.data[0] & 0x0000FFFF) | ((value as u32) << 16);
    }
}
