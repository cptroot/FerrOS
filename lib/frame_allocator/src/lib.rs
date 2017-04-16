#![no_std]

extern crate mem;

use ::mem::{Frame, PhysicalAddress};

/* Frame 1 is unused, frame 2 is for the AP Trampoline, and frame 3
 * is the AP Trampoline stack
 */
pub static mut FRAME_ALLOCATOR: FrameAllocator = FrameAllocator {
    next_frame: 4,
};

pub struct FrameAllocator {
    next_frame: usize,
}

impl FrameAllocator {
    pub fn get_frame(&mut self) -> Frame {
        let result = Frame::new(self.next_frame);
        self.next_frame += 1;
        result
    }

    pub fn get_multiple_frames(&mut self, num_frames: usize) -> Frame {
        let result = Frame::new(self.next_frame);
        self.next_frame += num_frames;
        result
    }
}
