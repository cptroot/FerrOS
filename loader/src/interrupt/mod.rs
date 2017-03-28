
mod idt;

use self::idt::InterruptDescriptor;
use self::idt::IDT;
use x86;
use core::mem;

pub fn intr_disable() {
    unsafe { x86::shared::irq::disable(); }
}

pub fn intr_enable() {
    unsafe { x86::shared::irq::enable(); }
}

const INTR_CNT: usize = 256;

// This works with the null pointer optimization, so that we're
// just storing nullable pointers
static mut INTERRUPT_HANDLERS:
    [Option<InterruptHandlerFunction>; INTR_CNT] =
    [None; INTR_CNT];

pub type InterruptHandlerFunction = extern fn() -> !;

fn make_intr_gate(f:InterruptHandlerFunction) -> InterruptDescriptor {
    let mut result = InterruptDescriptor::new([0; 4]);
    let ptr: usize = f as usize;
    result.set_offset_high(((ptr & 0xFFFFFFFF00000000) >> 32) as u32);
    result.set_offset_med(((ptr & 0x00000000FFFF0000) >> 16) as u16);
    result.set_offset_low(((ptr & 0x000000000000FFFF) >> 0) as u16);
    result.set_present_flag(true);
    //result.set_descriptor_privilege_level(0);
    result.set_gate_type(SystemDescriptorTypes::InterruptGate);
    //result.set_ist(0);
    result.set_segment_selector(x86::shared::segmentation::cs().bits());

    result
}

// Only call with a given interrupt from one thread at a time
// Easier is never call twice with a given interrupt
pub fn install_handler(f: InterruptHandlerFunction, interrupt: usize) {
    let descriptor = make_intr_gate(f);
    unsafe {
        INTERRUPT_HANDLERS[interrupt] = Some(f);
        IDT[interrupt] = descriptor;
    }
}

pub fn install_idt() {
    unsafe {
        let ptr = x86::shared::dtables::DescriptorTablePointer {
            limit: mem::size_of_val(&IDT) as u16 - 1,
            base: &IDT as *const _ as *const x86::bits64::irq::IdtEntry,
        };
        x86::shared::dtables::lidt(&ptr);
    }
}

#[repr(u8)]
pub enum SystemDescriptorTypes {
    _LDT = 2,
    _AvailableTSS = 9,
    _BusyTSS = 11,
    _CallGate = 12,
    InterruptGate = 14,
    _TrapGate = 15,
}

