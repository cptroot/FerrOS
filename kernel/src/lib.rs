#![feature(lang_items)]
#![feature(asm)]
#![feature(plugin)]
#![feature(compiler_builtins_lib)]
#![feature(const_fn)]
#![feature(naked_functions)]
#![no_std]

extern crate compiler_builtins;
#[macro_use]
extern crate lazy_static;

// Pulls in memset, memcmp, memcpy
extern crate rlibc;

// Pulls in llvm intrinsics
//extern crate compiler_builtins;

#[macro_use]
extern crate x86;

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
mod interrupt;

mod apic;

fn install_handlers() {
    for i in 0..HANDLERS.len() {
        if let Some(handler) = HANDLERS[i] {
            interrupt::install_handler(handler, i);
        }
    }
    interrupt::install_handler(timer_handler, 0x20);
    interrupt::intr_disable();
    interrupt::install_idt();
    interrupt::intr_enable();
}

static mut LAPIC_REGISTERS: Option<apic::LapicRegisters> = None;

static mut testing: i64 = 32;

/// This is the entry point for the rust language part of the
/// OS. At this point all UEFI code can still be run, and
/// we haven't yet exited boot services
#[no_mangle]
pub extern fn kernel_entry(system_table:&gnu_efi::api::SystemTable, mut frame_allocator: falloc::FrameAllocator, mut page_table: page_table::PageTable) -> ! {
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
            println!("{:08x}", lapic_registers.get_lvt_timer_register());
            println!("{:08x}", lapic_registers.get_timer_initial_count_register());

            lapic_registers.set_lvt_timer_register(apic::TimerMode::Periodic, false, 0x20);
            lapic_registers.set_timer_initial_count_register(8000000);
        }
    }

    // Testing LAPIC
    unsafe {
        while testing > 0{
        }
    }

    /*
    // Initialize the GDT
    unsafe {
        use x86::shared::segmentation;
        use x86::shared::segmentation::{SegmentDescriptor, Type};
        use x86::shared::segmentation::{CODE_READ, DATA_WRITE};
        use x86::shared::PrivilegeLevel;
        use x86::shared::dtables::DescriptorTablePointer;
        let gdt_page = mem::Page::new(0x500);
        let mut gdt_address: mem::VirtualAddress = gdt_page.into();

        let segment_descriptors: &mut [SegmentDescriptor] =
            core::slice::from_raw_parts_mut(
                gdt_address.as_mut_ptr() as *mut SegmentDescriptor, 512);
        segment_descriptors[0] = SegmentDescriptor::NULL;
        segment_descriptors[1] = SegmentDescriptor::new(0, 0, Type::Code(CODE_READ), false, PrivilegeLevel::Ring0);
        segment_descriptors[2] = SegmentDescriptor::new(0, 0, Type::Data(DATA_WRITE), false, PrivilegeLevel::Ring0);
        segment_descriptors[3] = SegmentDescriptor::new(0, 0, Type::Code(CODE_READ), false, PrivilegeLevel::Ring3);
        segment_descriptors[4] = SegmentDescriptor::new(0, 0, Type::Data(DATA_WRITE), false, PrivilegeLevel::Ring3);
        segment_descriptors[5] = SegmentDescriptor::new(0, 0, Type::Code(CODE_READ), false, PrivilegeLevel::Ring0);
        let gdt: DescriptorTablePointer<SegmentDescriptor> = DescriptorTablePointer::new_gdtp(segment_descriptors);

        x86::shared::dtables::lgdt(&gdt);

        /*
        asm!("\
                pushq $$0x8
                lea 0x3(%rip), %rax
                pushq %rax
                lretq
                1: nop
                " : );
                */

        //let data_selector = segmentation::SegmentSelector::new(2, PrivilegeLevel::Ring0);
        //segmentation::load_ss(data_selector);
    }*/

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
        asm!("hlt");
    }
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}

const SIZE_OF_INTERRUPT_STACK_PUSH: i8 = 3;

macro_rules! enter_interrupt {
    () => {
        asm!("\
            push %rbp
            mov %rsp, %rbp
            push %rax
            sub $$0x10, %rsp
        ");
    };
}

macro_rules! exit_interrupt {
    () => {
        asm!("\
            add $$0x10, %rsp
            pop %rax
            pop %rbp
        ");
        asm!("\
            iretq
        ");
        unreachable!();
    }
}

macro_rules! handler {
    (error_code:$code:ty, $fn_name:ident, $function:ident) => {
        #[naked]
        extern fn $fn_name() -> ! {
            let error_code_usize: usize;

            unsafe {
                // Move the error code down the stack
                // 0x18 is calculated by
                // (SIZE_OF_INTERUPT_STACK_PUSH - 2) * 0x8
                asm!("\
                    push %rax
                    mov 0x8(%rsp), %rax
                    mov %rax, -0x18(%rsp)
                    pop %rax
                    add $$0x8, %rsp
                ");

                enter_interrupt!();
                asm!("\
                    mov rdi, [rsp - 0x8]
                    call $0
                    " : : "i"($function as extern "C" fn($code)) : "rdi" : "intel"
                );
                exit_interrupt!();
            }
        }
    };
    ($fn_name:ident, $function:ident) => {
        #[naked]
        extern fn $fn_name() -> ! {
            unsafe {
                enter_interrupt!();
            }
            $function();
            unsafe {
                exit_interrupt!();
            }
        }
    };
}

extern fn error_code_handler(error_code: u64) {
    //println!("hello from error_code_handler");
    unsafe {
    }
}

extern fn handler() {
    println!("exception");
    unsafe {
        panic!("exception");
    }
}

struct PageFaultErrorCode {
    error: u64,
}

extern fn page_fault_fn(error_code: PageFaultErrorCode) {
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

extern fn timer() {
    println!("in timer");
    unsafe {
        testing -= 1;
        LAPIC_REGISTERS.as_mut().unwrap().eoi();
    }
}

handler!(divide_by_zero_handler, handler);
handler!(debug_exception_handler, handler);
handler!(breakpoint_handler, handler);
handler!(overflow_handler, handler);
handler!(bound_handler, handler);
handler!(invalid_opcode_handler, handler);
handler!(missing_fpu_handler, handler);
handler!(double_fault_handler, handler);
handler!(invalid_tss_handler, handler);
handler!(segment_not_present, handler);
handler!(stack_segment_fault_handler, handler);
handler!(general_protection_handler, handler);
handler!(error_code:PageFaultErrorCode, page_fault_handler, page_fault_fn);
handler!(fpu_error_handler, handler);
handler!(alignment_check_handler, handler);
handler!(machine_check_handler, handler);
handler!(simd_exception_handler, handler);
handler!(virtualization_exception_handler, handler);

handler!(timer_handler, timer);

static HANDLERS: [Option<extern fn() -> !>; 21] = [
    Some(divide_by_zero_handler),
    Some(debug_exception_handler),
    None, //Some(handler!(nmi_interrupt_handler, handler)),
    Some(breakpoint_handler),
    Some(overflow_handler),
    Some(bound_handler),
    Some(invalid_opcode_handler),
    Some(missing_fpu_handler),
    Some(double_fault_handler),
    None,
    Some(invalid_tss_handler),
    Some(segment_not_present),
    Some(stack_segment_fault_handler),
    Some(general_protection_handler),
    Some(page_fault_handler),
    None, //Intel Reseved
    Some(fpu_error_handler),
    Some(alignment_check_handler),
    Some(machine_check_handler),
    Some(simd_exception_handler),
    Some(virtualization_exception_handler),
];


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
