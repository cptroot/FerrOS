use x86;

use apic;
/// Returns whether or not the CPU has a LAPIC
pub fn cpuid_lapic_present() -> bool {
    unsafe {
        let out:u32;
        asm!(
            "\
            movl $$0x1, %eax
            cpuid
            mov %edx, $0
            " :
            "=r"(out));
        ((out >> 9) & 0x1) == 0x1
    }
}

/// Returns whether or not the CPU has support for X2APIC
pub fn cpuid_x2apic_enabled() -> bool {
    unsafe {
        let ecx:u32;
        asm!(
            "\
            movl $$0x1, %eax
            cpuid
            " :
            "={ecx}"(ecx));
        ((ecx >> 21) & 0x1) == 0x1
    }
}

/// Returns whether the LAPIC is enabled, and whether
/// it is in bootstrap or extended mode
pub fn cpuid_lapic_enabled() -> (apic::ApicRegisters, bool, bool, bool) {
    unsafe {
        let high:u32;
        let low:u32;
        asm!(
            "\
            mov $$0x1B, %ecx
            rdmsr
            " :
            "={edx}"(high), "={eax}"(low));
        let address:u64 = (((high & 0x0000000F) as u64) << 32) | ((low & 0xFFFFF000) as u64);
        let bootstrap = (low >> 8) & 0x1 == 0x1;
        let enabled =   (low >> 11) & 0x1 == 0x1;
        let extended = (low >> 10) & 0x1 == 0x1;
        (apic::ApicRegisters::new(address as *mut _), bootstrap, enabled, extended)
    }
}

pub fn get_segment_selector() -> u16 {
    unsafe {
        let cs:u16;
        asm!("\
            mov %cs, %ax
            " :
            "={ax}"(cs));
        cs
    }
}

#[allow(dead_code)]
/// Retrieves the global description table register
pub fn sgdt() -> x86::shared::dtables::DescriptorTablePointer<x86::shared::segmentation::SegmentDescriptor> {
    unsafe {
        use core::mem;
        let gdt_ptr: x86::shared::dtables::DescriptorTablePointer<x86::shared::segmentation::SegmentDescriptor> = mem::uninitialized();
        asm!(
            "\
            sgdt $0
            " :
            "=*m"(&gdt_ptr));
        return gdt_ptr;
    }
}

#[allow(dead_code)]
/// Retrieves the global description table register
pub fn sidt() -> x86::shared::dtables::DescriptorTablePointer<x86::bits64::irq::IdtEntry> {
    unsafe {
        use core::mem;
        let idt_ptr: x86::shared::dtables::DescriptorTablePointer<x86::bits64::irq::IdtEntry> = mem::uninitialized();
        asm!(
            "\
            sidt $0
            " :
            "=*m"(&idt_ptr));
        return idt_ptr;
    }
}
