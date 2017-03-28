use core::mem;

/// Bindings to efilib helper functions

#[link(name = "gnuefi")]
extern {
    fn Print(fmt: *const u16, ...) -> usize;
    fn LibMemoryMap(no_entries: *const usize,
                    map_key: *const usize,
                    descriptor_size: *const usize,
                    descriptor_version: *const u32)
        -> *const ::def::MemoryDescriptor;
}

pub fn print_wide(wide_fmt:&[u16]) {
    unsafe {
        Print(wide_fmt.as_ptr());
    }
}

/// Prints a character string to stdout
pub fn print(fmt:&'static str) {
    assert!(fmt.len() < 64);
    let mut wide_fmt = [0u16; 64];
    for (i, c) in fmt.bytes().enumerate() {
        wide_fmt[i] = c as u16;
    }
    unsafe {
        Print(wide_fmt.as_ptr());
    }
}

/// Prints an int to stdout
pub fn print_int(value: isize) {
    let mut wide_fmt = [0u16; 64];
    for (i, c) in "%d".bytes().enumerate() {
        wide_fmt[i] = c as u16;
    }
    unsafe {
        Print(wide_fmt.as_ptr(), value);
    }
}

/// Prints a hex value to stdout
pub fn print_hex(value: usize) {
    let mut wide_fmt = [0u16; 64];
    for (i, c) in "%x".bytes().enumerate() {
        wide_fmt[i] = c as u16;
    }
    unsafe {
        Print(wide_fmt.as_ptr(), value);
    }
}

pub fn print_device_path(
        boot_services: &::api::services::BootServices,
        device_path: &::api::protocol::DevicePathProtocol) {
    let handles = boot_services.retrieve_handles_with_protocol::<::api::protocol::DevicePathToTextProtocol>().unwrap();
    let device_path_to_text_protocol:
            *mut ::api::protocol::DevicePathToTextProtocol =
        boot_services.retrieve_protocol_from_handle(handles.get(0).as_ref().unwrap()).unwrap();
    let text: *const u16 = unsafe {
        (*device_path_to_text_protocol).device_path_to_text(device_path)
    };

    unsafe {
        Print(text);
        print("\n");
    }
}

/// Retrieves the current memory map
pub fn lib_memory_map() -> (::def::MemoryDescriptors, usize) {
    unsafe {
        let mut no_entries = mem::uninitialized();
        let mut map_key = mem::uninitialized();
        let mut descriptor_size = mem::uninitialized();
        let mut descriptor_version = mem::uninitialized();
        let memory_map_ptr = LibMemoryMap(
            &mut no_entries,
            &mut map_key,
            &mut descriptor_size,
            &mut descriptor_version);
        (::def::MemoryDescriptors::new(memory_map_ptr, no_entries, descriptor_size), map_key)
    }
}
