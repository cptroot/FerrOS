#![feature(lang_items)]
#![feature(asm)]
#![feature(plugin)]
#![feature(compiler_builtins_lib)]
#![feature(const_fn)]
#![feature(abi_x86_interrupt)]
#![no_std]

extern crate compiler_builtins;
#[macro_use]
extern crate lazy_static;

// Pulls in memset, memcmp, memcpy
extern crate rlibc;

// Pulls in llvm intrinsics
//extern crate compiler_builtins;

extern crate x86;

extern crate x86_64;

extern crate mem;

// management of efi and acpi functions and tables
extern crate gnu_efi;

// Code for writing to serial output
#[macro_use]
extern crate serial;

extern crate frame_allocator as falloc;

extern crate page_table;

// bindings to cpuid
mod asm_routines;

// Code for modifying and using IDT
//mod interrupt;

mod apic;

lazy_static! {
    static ref IDT: x86_64::structures::idt::Idt = {
        let mut idt = x86_64::structures::idt::Idt::new();
        idt.divide_by_zero.set_handler_fn(divide_by_zero_handler);
        idt.debug.set_handler_fn(debug_exception_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(missing_fpu_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point.set_handler_fn(fpu_error_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_exception_handler);
        idt.virtualization.set_handler_fn(virtualization_exception_handler);
        idt.interrupts[0].set_handler_fn(timer_handler);
        idt.interrupts[0xdf].set_handler_fn(spurious_interrupt_handler);
        idt
    };
}

static mut LAPIC_REGISTERS: Option<apic::LapicRegisters> = None;

static mut testing: i64 = 32;

/// This is the entry point for the rust language part of the
/// OS. At this point all UEFI code can still be run, and
/// we haven't yet exited boot services
#[no_mangle]
pub extern fn kernel_entry(system_table:&gnu_efi::api::SystemTable, mut frame_allocator: falloc::FrameAllocator, mut page_table: page_table::PageTable) -> ! {
    // Initialize the GDT
    unsafe {
        use x86::shared::segmentation::{SegmentDescriptor};
        use x86::shared::dtables::DescriptorTablePointer;
        let gdt_frame = frame_allocator.get_frame();
        let gdt_frame_num: usize = gdt_frame.into();
        let gdt_page = ::mem::Page::new(gdt_frame_num);
        let mut gdt_address: mem::VirtualAddress = gdt_page.into();

        let segment_descriptors: &mut [u64] =
            core::slice::from_raw_parts_mut(
                gdt_address.as_mut_ptr() as *mut u64, 512);
        segment_descriptors[0] = 0;
        segment_descriptors[1] = 0;
        segment_descriptors[2] = 0x0020_9a00_0000_0000;
        segment_descriptors[3] = 0x0080_9200_0000_0000;
        segment_descriptors[4] = 0x0020_fa00_0000_0000;
        segment_descriptors[5] = 0x0080_f200_0000_0000;
        let gdt: DescriptorTablePointer<SegmentDescriptor> = DescriptorTablePointer::new_gdtp(::core::mem::transmute(segment_descriptors));

        x86::shared::dtables::lgdt(&gdt);

        asm!("\
                pushq $$0x10
                lea 0x3(%rip), %rax
                pushq %rax
                lretq
                1: nop
                " : );

        asm!("\
                mov $$0x0, %eax
                mov %ax, %ds
                mov %ax, %es
                mov %ax, %fs
                mov %ax, %gs
                mov %ax, %ss
                " : );

    }
    // Override IDT
    install_handlers();

    println!("");

    //divide_by_zero();

    /*unsafe {
        let ptr: *mut u8 = 0x0 as *mut u8;
        *ptr = 0;
    }*/

    unsafe {
        core::mem::replace(&mut falloc::FRAME_ALLOCATOR, frame_allocator);
    }

    if asm_routines::cpuid_lapic_present() {
        println!("lapic present");
    }

    if asm_routines::cpuid_x2apic_enabled() {
        println!("x2apic supported");
    }

    // Get properties of the LAPIC
    let (mut lapic_registers, bootstrap, enabled, extended) =
        asm_routines::cpuid_lapic_enabled();
    if bootstrap {
        println!("lapic is in bootstrap mode");
    }
    if enabled {
        println!("lapic is enabled");
    }
    if extended {
        println!("x2apic is enabled");
    }

    // Get the RSDP from the ACPI table. Relies on
    // vendor table from UEFI
    let rsdp = gnu_efi::acpi::get_rsdp(system_table);

    if rsdp.verify() {
        println!("Found valid RSDP");
    }

    // Verify the extended system description table.
    if rsdp.xsdt.verify() {
        println!("Found valid XSDT");
    }

    // Find the Multiple Apic Description Table
    if let Some(madt) = rsdp.xsdt.find_madt() {
        println!("Found valid MADT");

        println!("Enumerated MADT types:");
        for header in madt.controllers() {
            print!("type: {:?}", header.structure_type);

            match header.structure_type {
                gnu_efi::acpi::ApicStructureType::InterruptSourceOverride => {
                    let iso = header.to_interrupt_source_override();
                    print!(" source: {}", iso.source);
                    print!(" interrupt: {}", iso.global_system_interrupt);
                },
                _ => {
                }
            }
            println!("");
        }
    }

    // LAPIC configuration
    // Page in LAPIC
    lapic_registers.page_in(&mut page_table);

    println!("lapic APIC ID: {:x}", lapic_registers.get_apic_id_register());
    unsafe {
        let address: *mut u32 = 0x3100 as *mut u32;
        *address = page_table.physical_address();
        let address: *mut u64 = 0x3200 as *mut u64;
        *address = (ap_bootstrap) as u64;
        lapic_registers.send_startup_ipi();
    }
    unsafe {
        LAPIC_REGISTERS = Some(lapic_registers);
        if let Some(ref mut lapic_registers) = LAPIC_REGISTERS {
            lapic_registers.enable_lapic(0xff);

            println!("{:08x}", lapic_registers.get_lvt_timer_register());
            println!("{:08x}", lapic_registers.get_timer_initial_count_register());

            lapic_registers.set_lvt_timer_register(apic::TimerMode::Periodic, false, 0x20);
            //lapic_registers.set_timer_initial_count_register(8000000);
        }
    }

    // Testing LAPIC
    unsafe {
        while testing > 0{
        }
    }


    // Shutdown the computer
    system_table.runtime_services.reset_system(
        gnu_efi::api::ResetType::ResetShutdown,
        gnu_efi::def::Status::Success,
        0,
        core::ptr::null());
}

fn ap_bootstrap() {
    println!("hello from processor 2");
    unsafe {
        x86_64::instructions::interrupts::disable();
        IDT.load();
        x86_64::instructions::interrupts::enable();
    }
    println!("idt installed");

    unsafe {
        if let Some(ref mut lapic_registers) = LAPIC_REGISTERS {
            lapic_registers.enable_lapic(0xff);
            println!("lapic APIC ID: {:x}", lapic_registers.get_apic_id_register());

            println!("{:08x}", lapic_registers.get_lvt_timer_register());
            println!("{:08x}", lapic_registers.get_timer_initial_count_register());

            lapic_registers.set_lvt_timer_register(apic::TimerMode::Periodic, false, 0x20);
            lapic_registers.set_timer_initial_count_register(8000000);

            println!("{:08x}", lapic_registers.get_lvt_timer_register());
        }
    }

    unsafe {
        while testing > 0{
        }
    }

    unsafe {
        asm!("hlt");
    }
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}

use x86_64::structures::idt::{ExceptionStackFrame, PageFaultErrorCode};

extern "x86-interrupt" fn page_fault_handler(stack_frame: &mut ExceptionStackFrame, error_code: PageFaultErrorCode) {
    let cr2: usize = unsafe {
        let result: usize;
        asm!("\
            mov %cr2, $0
            " : "=r"(result));
        result
    };
    print_something_else(cr2);
}

fn print_something_else(cr2: usize) {
    println!("Page fault address: {:x}", cr2);
}

extern "x86-interrupt" fn timer_handler(stack_frame: &mut ExceptionStackFrame) {
    println!("in timer");
    unsafe {
        testing -= 1;
        LAPIC_REGISTERS.as_mut().unwrap().eoi();
    }
}

macro_rules! unhandled_exception_handler {
    ( $fn_name:ident ) => {
        extern "x86-interrupt" fn $fn_name(_: &mut ExceptionStackFrame) {
            println!("unhandled exception: {}", stringify!($fn_name));
        }
    };
    ( error $fn_name:ident ) => {
        extern "x86-interrupt" fn $fn_name(_: &mut ExceptionStackFrame, _:u64) {
            println!("unhandled exception: {}", stringify!($fn_name));
        }
    };
}

fn install_handlers() {
    unsafe {
        x86_64::instructions::interrupts::disable();
        IDT.load();
        x86_64::instructions::interrupts::enable();
    }
}

unhandled_exception_handler!(divide_by_zero_handler);
unhandled_exception_handler!(debug_exception_handler);
unhandled_exception_handler!(breakpoint_handler);
unhandled_exception_handler!(overflow_handler);
unhandled_exception_handler!(bound_handler);
unhandled_exception_handler!(invalid_opcode_handler);
unhandled_exception_handler!(missing_fpu_handler);
unhandled_exception_handler!(error double_fault_handler);
unhandled_exception_handler!(error invalid_tss_handler);
unhandled_exception_handler!(error segment_not_present);
unhandled_exception_handler!(error stack_segment_fault_handler);
unhandled_exception_handler!(error general_protection_handler);
unhandled_exception_handler!(fpu_error_handler);
unhandled_exception_handler!(error alignment_check_handler);
unhandled_exception_handler!(machine_check_handler);
unhandled_exception_handler!(simd_exception_handler);
unhandled_exception_handler!(virtualization_exception_handler);

unhandled_exception_handler!(spurious_interrupt_handler);

/// Special functions to make the compiler happy. Maybe
/// eventually these will be used to support runtime
/// unwinding of panics.
#[cfg(not(test))]
#[no_mangle]
#[lang = "eh_personality"] extern fn rust_eh_personality() {}
#[cfg(not(test))]
#[lang = "panic_fmt"]
#[no_mangle]
extern fn rust_begin_unwind(
        msg: core::fmt::Arguments,
        file: &'static str,
        line: u32) -> ! {
    unsafe {
        use core::fmt::Write;
        let mut writer = serial::SerialWriter::new();
        let _ = writer.write_fmt(msg);
        let _ = writer.write_fmt(format_args!(" in file {} on line {}\n", file, line));
    }
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}
