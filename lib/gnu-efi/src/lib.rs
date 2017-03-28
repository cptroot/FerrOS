#![feature(try_from)]
#![feature(nonzero)]
#![no_std]

extern crate mem;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

/// Contains a large enum used by def. Might rearrange the module
/// structure at some point
mod err;

/// Corresponds to efidef.h
pub mod def;

/// Takes in a C string and computes the length.
/// Used to create Rust slices from C strings
pub fn strlen(ptr:*const u8) -> usize {
    let mut iter = ptr;
    unsafe {
        while *iter != 0 {
            iter = iter.offset(1);
        }
    }
    return (iter as usize) - (ptr as usize);
}

/// Corresponds to efibind.h
pub mod bind;
/// Corresponds to efibind.h
pub mod efilib;
/// Corresponds to efibind.h
pub mod api;
/// ACPI bindings and table definitions
pub mod acpi;


