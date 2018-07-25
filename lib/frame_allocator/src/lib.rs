#![no_std]
#![feature(const_fn)]

extern crate mem;

use ::mem::{Frame, PhysicalAddress};
use ::core::ops::DerefMut;

pub struct FrameAllocator {
    next_frame: usize,
}

impl FrameAllocator {
    pub const fn new() -> FrameAllocator {
        FrameAllocator {
            next_frame: 5,
        }
    }

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

pub trait FrameGetter {
    type FrameLock: DerefMut<Target = FrameAllocator>;
    fn get_frame_allocator() -> Self::FrameLock;
}
