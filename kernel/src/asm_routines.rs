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
pub fn cpuid_lapic_enabled() -> (apic::LapicRegisters, bool, bool, bool) {
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
        (apic::LapicRegisters::new(address as *mut _), bootstrap, enabled, extended)
    }
}
