#![feature(lang_items)]
#![feature(asm)]
#![feature(abi_x86_interrupt)]
#![feature(plugin)]
#![feature(unique)]
#![feature(compiler_builtins_lib)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(const_unsafe_cell_new)]
#![no_std]

extern crate compiler_builtins;

// Pulls in memset, memcmp, memcpy
extern crate rlibc;

// Pulls in llvm intrinsics
//extern crate compiler_builtins;

extern crate x86;
extern crate x86_64;

extern crate spin;
extern crate atomic_ring_buffer;

extern crate mem;

// management of efi and acpi functions and tables
extern crate gnu_efi;

// Code for writing to serial output
#[macro_use]
extern crate serial;

extern crate frame_allocator as falloc;

extern crate page_table;

mod palloc;

// bindings to cpuid
mod asm_routines;

mod once_mut;

// Code for modifying and using IDT
//mod interrupt;

mod interrupts;

mod threads;

mod apic;

mod initialize_once;

mod per_cpu;

use core::sync::atomic::AtomicUsize;

use interrupts::{SingleExecution, InterruptGuard};
use initialize_once::InitializeOnce;

static IDT: InitializeOnce<x86_64::structures::idt::Idt> = InitializeOnce::new();
static PAGE_TABLE: InitializeOnce<spin::Mutex<page_table::PageTable>> = InitializeOnce::new();
struct FramePlace;
impl ::falloc::FrameGetter for FramePlace {
    type FrameLock = spin::MutexGuard<'static, ::falloc::FrameAllocator>;
    fn get_frame_allocator() -> Self::FrameLock {
        FRAME_ALLOCATOR.lock()
    }
}
static FRAME_ALLOCATOR: InitializeOnce<spin::Mutex<::falloc::FrameAllocator>> = InitializeOnce::new();

static mut LAPIC_REGISTERS: Option<apic::LapicRegisters> = None;

static PROCESSOR_COUNTER: AtomicUsize = AtomicUsize::new(0);

/*
pub fn initialize_statics(mut page_table: page_table::PageTable) {
    PAGE_TABLE.call_once(spin::Mutex::new(page_table));
}
*/

pub fn initialize_idt(single_execution: &SingleExecution) {
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

    IDT.initialize(single_execution, idt);
}

/// This is the entry point for the rust language part of the
/// OS. At this point all UEFI code can still be run, and
/// we haven't yet exited boot services
#[no_mangle]
pub extern fn kernel_entry(
    system_table:&gnu_efi::api::SystemTable,
    mut frame_allocator: falloc::FrameAllocator,
    page_table: page_table::PageTable)
        -> ! {
    let single_execution = unsafe {
        SingleExecution::new()
    };
    let interrupt_guard = interrupts::disable();

    PAGE_TABLE.initialize(&single_execution, ::spin::Mutex::new(page_table));
    FRAME_ALLOCATOR.initialize(&single_execution, ::spin::Mutex::new(frame_allocator));

    // Initialize the GDT
    unsafe {
        use x86::shared::segmentation::{SegmentDescriptor};
        use x86::shared::dtables::DescriptorTablePointer;
        let gdt_frame = FRAME_ALLOCATOR.lock().get_frame();
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
    initialize_idt(&single_execution);
    install_handlers();

    //println!("");

    //divide_by_zero();

    //page_fault();

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
    lapic_registers.page_in(&mut *PAGE_TABLE.lock());

    println!("lapic APIC ID: {:x}", lapic_registers.get_apic_id_register());

    // Initialize Per Cpu Data Blocks
    unsafe {
        ::per_cpu::initialize_per_cpu(&single_execution, 2);
    }

    // Send startup IPI
    unsafe {
        let address: *mut u32 = 0x3100 as *mut u32;
        *address = PAGE_TABLE.lock().physical_address();
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
            lapic_registers.set_timer_initial_count_register(8000000);
        }
    }

    // Set the Processor ID
    {
        // Get a new processor ID
        let processor_id = PROCESSOR_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        // Store the Processor ID into rdtscp MSR
        unsafe {
            ::per_cpu::write_processor_id(::per_cpu::ProcessorId::new(processor_id as u64));
        }
    }

    // Create the Idle thread for this processor
    let idle_thread = {
        let page = ::mem::Page::new(0x80_0040_0);
        let frame = FRAME_ALLOCATOR.lock().get_frame();

        PAGE_TABLE.lock().insert_page::<FramePlace>(frame, page, ::page_table::PageSize::FourKb);

        let page_vaddr: ::mem::VirtualAddress = page.into();
        let addr_usize: usize = page_vaddr.into();
        let thread: *mut threads::Thread = addr_usize as *mut threads::Thread;

        threads::Thread::new_in_place_no_insert(thread, true, threads::idle_thread);

        thread
    };
    ::per_cpu::retrieve_per_cpu(&interrupt_guard).set_idle_thread(&single_execution, idle_thread);

    let first_thread = {
        let page = ::mem::Page::new(0x80_0010_0);
        let frame = FRAME_ALLOCATOR.lock().get_frame();

        PAGE_TABLE.lock().insert_page::<FramePlace>(frame, page, ::page_table::PageSize::FourKb);

        let page_vaddr: ::mem::VirtualAddress = page.into();
        let addr_usize: usize = page_vaddr.into();
        let thread: *mut threads::Thread = addr_usize as *mut threads::Thread;

        threads::Thread::new_in_place_no_insert(thread, false, count_up);

        thread
    };

    {
        let page = ::mem::Page::new(0x80_0030_0);
        let frame = FRAME_ALLOCATOR.lock().get_frame();

        PAGE_TABLE.lock().insert_page::<FramePlace>(frame, page, ::page_table::PageSize::FourKb);

        let page_vaddr: ::mem::VirtualAddress = page.into();
        let addr_usize: usize = page_vaddr.into();
        let thread: *mut threads::Thread = addr_usize as *mut threads::Thread;

        threads::Thread::new_in_place(thread, up_down);

    }
    {
        let page = ::mem::Page::new(0x80_0050_0);
        let frame = FRAME_ALLOCATOR.lock().get_frame();

        PAGE_TABLE.lock().insert_page::<FramePlace>(frame, page, ::page_table::PageSize::FourKb);

        let page_vaddr: ::mem::VirtualAddress = page.into();
        let addr_usize: usize = page_vaddr.into();
        let thread: *mut threads::Thread = addr_usize as *mut threads::Thread;

        threads::Thread::new_in_place(thread, some_counting);

    }

    unsafe {
        if let Some(ref mut lapic_registers) = LAPIC_REGISTERS {
            lapic_registers.set_timer_initial_count_register(80000000);
        }
    }

    unsafe {
        (*first_thread).start_first_thread(single_execution);
    }

    /*
    // Shutdown the computer
    system_table.runtime_services.reset_system(
        gnu_efi::api::ResetType::ResetShutdown,
        gnu_efi::def::Status::Success,
        0,
        core::ptr::null());
    */
}

fn up_down() {
    let mut i = 0;
    loop {
        while i < 10000000 {
            i += 1;
        }
        println!("i goes up!");
        while i > 0 {
            i -= 1;
        }
        println!("i comes down!");
    }
}

fn count_up() {
    let mut i = 0;
    while i < 10000000 {
        if i == 9999999 {
            println!("i == 9999999");
            i = 0;
        } else {
            i += 1;
        }
    }
}

fn count_down() {
    let mut i = 10000000;
    while i != 0 {
        if i > 1 {
            i -= 1;
        } else {
            i = 10000000;
            println!("i == 1");
        }
    }
}

fn some_counting() {
    let mut i = 0;
    let mut j = 10;
    while j > 0 {
        i = 0;
        while i < 1000000 {
            i += 1;
        }
        j -= 1;
    }
}

fn ap_initialize_idt() {
    unsafe {
        x86_64::instructions::interrupts::disable();
        IDT.load();
        x86_64::instructions::interrupts::enable();
    }
    println!("idt installed");
}

fn ap_initialize_timer() {
    unsafe {
        if let Some(ref mut lapic_registers) = LAPIC_REGISTERS {
            lapic_registers.enable_lapic(0xff);
            println!("lapic APIC ID: {:x}", lapic_registers.get_apic_id_register());

            println!("{:08x}", lapic_registers.get_lvt_timer_register());
            println!("{:08x}", lapic_registers.get_timer_initial_count_register());

            lapic_registers.set_lvt_timer_register(apic::TimerMode::Periodic, false, 0x20);
            lapic_registers.set_timer_initial_count_register(80000000);

            println!("{:08x}", lapic_registers.get_lvt_timer_register());
        }
    }
}

fn ap_initialize_thread(single_execution: &SingleExecution, interrupt_guard: &InterruptGuard) {
    let idle_thread = {
        let page = ::mem::Page::new(0x80_0020_0);
        let frame = FRAME_ALLOCATOR.lock().get_frame();

        PAGE_TABLE.lock().insert_page::<FramePlace>(frame, page, ::page_table::PageSize::FourKb);

        let page_vaddr: ::mem::VirtualAddress = page.into();
        let addr_usize: usize = page_vaddr.into();
        let thread: *mut threads::Thread = addr_usize as *mut threads::Thread;

        threads::Thread::new_in_place_no_insert(thread, true, threads::idle_thread);

        thread
    };
    ::per_cpu::retrieve_per_cpu(interrupt_guard).set_idle_thread(single_execution, idle_thread);
}

fn ap_start_scheduler(single_execution: SingleExecution, interrupt_guard: InterruptGuard) -> ! {
    let thread = *::per_cpu::retrieve_per_cpu(&interrupt_guard).get_idle_thread();
    ::core::mem::drop(interrupt_guard);
    unsafe {
        (*thread).start_first_thread(single_execution);
    }
}

fn ap_bootstrap() -> ! {
    println!("hello from processor 2");

    let interrupt_guard = interrupts::disable();
    let single_execution = unsafe {
        SingleExecution::new()
    };

    // Set the Processor ID
    {
        // Get a new processor ID
        let processor_id = PROCESSOR_COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        // Store the Processor ID into rdtscp MSR
        unsafe {
            ::per_cpu::write_processor_id(::per_cpu::ProcessorId::new(processor_id as u64));
        }
    }

    ap_initialize_idt();
    ap_initialize_timer();
    ap_initialize_thread(&single_execution, &interrupt_guard);
    ap_start_scheduler(single_execution, interrupt_guard);
}

fn divide_by_zero() {
    unsafe {
        asm!("mov dx, 0; div dx" ::: "ax", "dx" : "volatile", "intel")
    }
}

fn page_fault() {
    unsafe {
        asm!("movq $$0, %rax; movq (%rax), %rax")
    }
}

use x86_64::structures::idt::{ExceptionStackFrame, PageFaultErrorCode};

extern "x86-interrupt" fn page_fault_handler(stack_frame: &mut ExceptionStackFrame, _error_code: PageFaultErrorCode) {
    unsafe {
        use ::core::fmt::Write;
        let mut writer = serial::SerialWriter::new_init();
        if let Err(err) = writer.write_fmt(format_args!("stack_frame: {:?}\n", *stack_frame)) {
            panic!("{}", err);
        }
    }

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
    unsafe {
        use ::core::fmt::Write;
        let mut writer = serial::SerialWriter::new_init();
        if let Err(err) = writer.write_fmt(format_args!("Page fault address: {:x}", cr2)) {
            panic!("{}", err);
        }
    }
}

extern "x86-interrupt" fn timer_handler(stack_frame: &mut ExceptionStackFrame) {
    let interrupt_guard = unsafe {
        interrupts::InterruptGuard::new(interrupts::InterruptState::Disabled)
    };
    unsafe {
        use ::core::fmt::Write;
        let mut writer = serial::SerialWriter::new_init();
        if let Err(err) = writer.write_fmt(format_args!("in timer\n")) {
            panic!("{}", err);
        }
    }

    let thread = ::threads::current_thread(&interrupt_guard);
    let should_schedule = stack_frame.stack_pointer.0 > 0x8000000000 && (*thread).tick() == 0;
    unsafe {
        LAPIC_REGISTERS.as_mut().unwrap().eoi();
        if should_schedule {
            thread.schedule(&interrupt_guard, threads::Status::Ready);
        }
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
#[lang = "eh_personality"] pub extern fn rust_eh_personality() {}
#[cfg(not(test))]
#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_unwind(
        msg: core::fmt::Arguments,
        file: &'static str,
        line: u32) -> ! {
    unsafe {
        use core::fmt::Write;
        let mut writer = serial::SerialWriter::new();
        let _ = writer.write_fmt(format_args!("PANIC "));
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
