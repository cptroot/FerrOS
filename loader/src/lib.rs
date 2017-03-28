#![feature(lang_items)]
#![feature(asm)]
#![feature(plugin)]
#![feature(const_fn)]
#![feature(naked_functions)]
#![no_std]

// Pulls in memset, memcmp, memcpy
extern crate rlibc;

// Utility to modify the CR3 register
extern crate x86;

// Memory types
extern crate mem;

// management of efi and acpi functions and tables
extern crate gnu_efi;

// Polling serial implementation
#[macro_use]
extern crate serial;

extern crate page_table;

extern crate frame_allocator as falloc;

extern crate elf;

//mod palloc;

static mut INIT_RAM_PAGES: usize = 0;

struct StackData {
    elf_file: elf::File<'static>,
    page_table: page_table::PageTable,
    system_table: &'static gnu_efi::api::SystemTable,
}

static mut STACK_DATA_GLOBAL: Option<StackData> = None;

/// This is the entry point for the rust language part of the
/// OS. At this point all UEFI code can still be run, and
/// we haven't yet exited boot services
#[no_mangle]
pub extern fn rust_main(image_handle:gnu_efi::def::Handle,
                        system_table:&mut gnu_efi::api::SystemTable) -> ! {
    // Get all handles supporting simple_file_protocol
    let handles = system_table.boot_services.retrieve_handles_with_protocol::<gnu_efi::api::protocol::SimpleFileSystemProtocol>();

    let size = 512000;
    if let Ok(mut buffer) = system_table.boot_services.allocate_pages((size + 0x200) / 0x200) {
        // Retrieve the kernel efi file
        let file = handles.unwrap().iter().filter_map(|handle| -> Option<&mut gnu_efi::api::protocol::SimpleFileSystemProtocol> {
            // Retrieve the protocol based off of the handle, filtering
            // when the protocol doesn't exist
            system_table.boot_services.retrieve_protocol_from_handle(handle).ok()
        }).filter_map(|protocol| {
            // Open each found volume
            protocol.open_volume().ok()
        }).filter_map(|root_directory| {
            // Try to navigate to the kernel efi for each found volume
            root_directory.open("EFI\\OS\\KERNEL.EFI").ok()
        }).next();

        // Read the efi file into memory, and parse it into an elf
        // file structure
        let elf_kernel = file.and_then(|file| {
            // Read the efi file into memory
            (*file).read(size, buffer.get_mut_pointer()).ok().map(|file_buffer| {
                // Initialize the elf_structure
                elf::File::from_buffer(file_buffer)
            })
        });

        // Allocate the page for the new stack
        let mut new_stack_page = system_table.boot_services.allocate_pages(10).unwrap();
        //let mut new_gdt_page = system_table.boot_services.allocate_pages(1).unwrap();

        // Use efilib to get memory map, involves allocating from UEFI
        // because we don't have control of all memory yet
        let (memory_map, map_key) = gnu_efi::efilib::lib_memory_map();

        // Exit boot services. At this point the rust kernel
        // can do whatever it wants as long as it doesn't kill
        // the runtime services code
        system_table.boot_services.exit_boot_services(
            &image_handle,
            map_key);

        print_memory_map(&memory_map);

        if let Some(elf_file) = elf_kernel {
            println!("{:?}", elf_file.file_header());
            // Initialize page table
            let mut page_table = unsafe {
                page_table::PageTable::new(
                    falloc::FRAME_ALLOCATOR.get_frame())
            };

            {
                // Create mapping for existing code
                for memory_descriptor in &memory_map {
                    use ::gnu_efi::def::MemoryType;
                    let keep = match memory_descriptor.region_type {
                        MemoryType::LoaderCode => true,
                        MemoryType::LoaderData => true,
                        MemoryType::RuntimeServicesCode => true,
                        MemoryType::RuntimeServicesData => true,
                        MemoryType::ACPIMemoryNVS => true,
                        MemoryType::ACPIReclaimMemory => true,
                        MemoryType::PalCode => false,
                        _ => false,
                    };

                    if keep {
                        // Add pages for each page
                        let frame_start: mem::Frame = memory_descriptor.physical_start.into();
                        let frame_number: usize = frame_start.into();
                        let page_start: mem::Page = mem::Page::new(frame_number);
                        for offset in 0isize..memory_descriptor.number_of_pages as isize {
                            let new_page = page_start + mem::PageOffset::new(offset);
                            let new_frame = frame_start + mem::FrameOffset::new(offset);

                            page_table.insert_page(new_frame, new_page, page_table::PageSize::FourKb);
                        }
                    }
                }
            }

            // Add a mapping for the first init_ram_pages pages
            //
            unsafe {
                INIT_RAM_PAGES = 0x1000;

                let start_page: usize = 0;
                for offset in 1..INIT_RAM_PAGES {
                    page_table.insert_page(
                        mem::Frame::new(start_page + offset),
                        mem::Page::new(start_page + offset),
                        page_table::PageSize::FourKb);
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
                let segment_descriptors: &mut [SegmentDescriptor] = core::slice::from_raw_parts_mut(new_gdt_page.as_mut_ptr() as *mut SegmentDescriptor, 512);
                segment_descriptors[0] = SegmentDescriptor::NULL;
                segment_descriptors[1] = SegmentDescriptor::new(0, 0, Type::Code(CODE_READ), false, PrivilegeLevel::Ring0);
                segment_descriptors[2] = SegmentDescriptor::new(0, 0, Type::Data(DATA_WRITE), false, PrivilegeLevel::Ring0);
                segment_descriptors[3] = SegmentDescriptor::new(0, 0, Type::Code(CODE_READ), false, PrivilegeLevel::Ring3);
                segment_descriptors[4] = SegmentDescriptor::new(0, 0, Type::Data(DATA_WRITE), false, PrivilegeLevel::Ring3);
                let gdt: DescriptorTablePointer<SegmentDescriptor> = DescriptorTablePointer::new_gdtp(segment_descriptors);

                //x86::shared::dtables::lgdt(&gdt);

                /*#[repr(Packed)]
                struct LJmp {
                    selector: u16,
                    offset: u64,
                };

                let LJmp = 

                asm!("\
                        movw $$8, ax
                        ljmpq ax, next_instruction
                        next_instruction: nop
                        ");*/

                //let data_selector = segmentation::SegmentSelector::new(2, PrivilegeLevel::Ring0);
                //segmentation::load_ss(data_selector);
            }*/

            // Initialize a new stack
            unsafe {
                println!("saving stack variables to globals");
                // Save current stack variables to globals
                let stack_data = StackData {
                    elf_file: ::core::mem::transmute(elf_file),
                    page_table: page_table,
                    system_table: ::core::mem::transmute(system_table),
                };

                ::core::mem::replace(&mut STACK_DATA_GLOBAL, Some(stack_data));

                // Set stack to be a new ebp/esp
                let stack_address: *mut u8 = new_stack_page.as_mut_ptr().offset(0x9_000);
                asm!("mov $0, %rsp" :: "r" (stack_address as usize) : "memory");
                asm!("push $$0");
                asm!("push $$0");
                asm!("mov %rsp, %rbp");

                // Call into the new_stack function to reset local variables
                let StackData { elf_file, page_table, system_table } =
                    ::core::mem::replace(&mut STACK_DATA_GLOBAL, None).unwrap();
                new_stack(elf_file, page_table, system_table);
            }
        }
    }

    system_table.runtime_services.reset_system(
        gnu_efi::api::ResetType::ResetShutdown,
        gnu_efi::def::Status::Success,
        0,
        core::ptr::null());
}

fn new_stack(elf_file: elf::File, mut page_table: page_table::PageTable, system_table: &::gnu_efi::api::SystemTable) -> ! {
    // Load page tables
    page_table.load();

    // For section in section_headers
    for section_header in elf_file.section_headers() {
        println!("{:?}", section_header);
        match section_header.section_type {
            SectionType::NoBits | SectionType::ProgramBits => {
                let size_bytes = mem::VirtualAddressOffset::new(section_header.size as isize);
                let num_pages: mem::PageOffset = size_bytes.into();
                let num_pages_isize: isize = num_pages.into();
                let start_page: mem::Page = {
                    let start_virtual_address = mem::VirtualAddress::new(section_header.virtual_address as usize);
                    start_virtual_address.into()
                };
                let start_frame: mem::Frame = unsafe { falloc::FRAME_ALLOCATOR.get_multiple_frames(num_pages_isize as usize) };

                for offset in 0..num_pages_isize {
                    let page = start_page + mem::PageOffset::new(offset);
                    let frame = start_frame + mem::FrameOffset::new(offset);

                    println!("map page {:?} into frame {:?}", page, frame);

                    page_table.insert_page(frame, page, page_table::PageSize::FourKb);
                }
            },
            _ => {},
        }
        use ::elf::SectionType;
        // Copy section to destination
        unsafe {
            match section_header.section_type {
                SectionType::NoBits =>
                    { rlibc::memset(section_header.addr_ptr(), 0, section_header.size); },
                SectionType::ProgramBits =>
                    { rlibc::memcpy(section_header.addr_ptr(), section_header.offset_buffer(&elf_file).as_ptr(), section_header.size); },
                _ => {},
            }
        }
    }

    unsafe {
        let entry: extern fn(system_table:&gnu_efi::api::SystemTable, falloc::FrameAllocator, page_table::PageTable) -> ! =
            core::mem::transmute(elf_file.file_header().entry_ptr());

        run_kernel(entry, system_table, page_table);
    }
}

fn run_kernel(entry: extern fn(system_table:&gnu_efi::api::SystemTable, falloc::FrameAllocator, page_table::PageTable) -> !, system_table:&gnu_efi::api::SystemTable, page_table:page_table::PageTable) -> ! {
    // Jump to entry
    let frame_allocator = unsafe {
        core::mem::replace(
            &mut falloc::FRAME_ALLOCATOR,
            core::mem::uninitialized() )
    };
    entry(system_table,
          frame_allocator,
          page_table);
}

fn print_memory_map(memory_map: &gnu_efi::def::MemoryDescriptors) {
    for memory_descriptor in memory_map {
        let start_address: usize = memory_descriptor.physical_start.into();
        let end_address: u64 = memory_descriptor.number_of_pages * 0x1000 + start_address as u64;
        println!("Type: {:?}, number of pages: {}, address: {:x}, end address: {:x}",
            memory_descriptor.region_type,
            memory_descriptor.number_of_pages,
            start_address,
            end_address);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

/// Special functions to make the compiler happy. Maybe
/// eventually these will be used to support runtime
/// unwinding of panics.
#[cfg(not(test))]
#[lang = "eh_personality"] extern fn eh_personality() {}
#[cfg(not(test))]
#[lang = "panic_fmt"]
extern fn panic_fmt(
        msg: core::fmt::Arguments,
        file: &'static str,
        line: u32) -> ! {
    println!("PANIC:");
    println!("{}", msg);

    println!("In {}:{}", file, line);
    loop {}
}
