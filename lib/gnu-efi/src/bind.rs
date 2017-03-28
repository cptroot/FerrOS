
use core::mem;
use ::def;

/// This trait allows type safety when calling uefi functions
/// All parameters passed into uefi functions must be
/// convertible to usize
pub trait EfiParameter {
    fn as_usize(&self) -> usize;
}

pub trait EfiOutput {
    fn from_usize(output:usize) -> Self;
}

impl EfiParameter for usize {
    fn as_usize(&self) -> usize {
        *self
    }
}

impl EfiParameter for u64 {
    fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl EfiParameter for bool {
    fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl<T> EfiParameter for *const T {
    fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl<T> EfiParameter for *mut T {
    fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl<'a, T> EfiParameter for &'a T {
    fn as_usize(&self) -> usize {
        let ptr: *const T = *self;
        ptr as usize
    }
}

impl<'a, T> EfiParameter for &'a mut T {
    fn as_usize(&self) -> usize {
        let ptr: *const T = *self;
        ptr as usize
    }
}

impl EfiParameter for def::Status {
    fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl EfiParameter for def::MemoryType {
    fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl EfiParameter for def::AllocateType {
    fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl EfiParameter for ::api::types::LocateSearchType {
    fn as_usize(&self) -> usize {
        *self as usize
    }
}

impl EfiOutput for ::def::Status {
    fn from_usize(integer: usize) -> ::def::Status {
        unsafe {
            mem::transmute(integer)
        }
    }
}

impl<T> EfiOutput for *const T {
    fn from_usize(integer: usize) -> *const T {
        integer as *const T
    }
}

/// Linking to the gnuefi efi_call functions
#[link(name = "gnuefi")]
extern "C" {
    fn efi_call1(func:extern fn(a:usize), a:usize) -> usize;
    fn efi_call2(func:extern fn(a:usize, b:usize), a:usize, b:usize) -> usize;
    fn efi_call3(func:extern fn(a:usize, b:usize, c:usize), a:usize, b:usize, c:usize) -> usize;
    fn efi_call4(func:extern fn(a:usize, b:usize, c:usize, d:usize), a:usize, b:usize, c:usize, d:usize) -> usize;
    fn efi_call5(func:extern fn(a:usize, b:usize, c:usize, d:usize, e:usize), a:usize, b:usize, c:usize, d:usize, e:usize) -> usize;
}

/// Rust safe functions that call the unsafe efi_call functions
/// These functions ensure type safety through type bounds on
/// the functions that you can call.

pub fn safe_efi_call1<U, Z>(f:extern fn (U) -> Z,
                        u: U)
    -> Z
        where U: EfiParameter,
              Z: EfiOutput,
{
    unsafe {
        EfiOutput::from_usize(efi_call1(
            mem::transmute(f),
            u.as_usize()))
    }
}

pub fn safe_efi_call2<U, V, Z>(f:extern fn(U, V) -> Z,
                        u:U, v:V)
    -> Z
        where U: EfiParameter,
              V: EfiParameter,
              Z: EfiOutput,
{
    unsafe {
        EfiOutput::from_usize(efi_call2(
            mem::transmute(f),
            u.as_usize(),
            v.as_usize()))
    }
}

pub fn safe_efi_call3<U, V, W, Z>(f:extern fn(U, V, W) -> Z,
                              u:U, v:V, w:W)
    -> Z
        where U: EfiParameter,
              V: EfiParameter,
              W: EfiParameter,
              Z: EfiOutput,
{
    unsafe {
        EfiOutput::from_usize(efi_call3(
            mem::transmute(f),
            u.as_usize(),
            v.as_usize(),
            w.as_usize()))
    }
}

pub fn safe_efi_call4<U, V, W, X, Z>(f:extern fn(U, V, W, X) -> Z,
                              u:U, v:V, w:W, x:X)
    -> Z
        where U: EfiParameter,
              V: EfiParameter,
              W: EfiParameter,
              X: EfiParameter,
              Z: EfiOutput,
{
    unsafe {
        EfiOutput::from_usize(efi_call4(
            mem::transmute(f),
            u.as_usize(),
            v.as_usize(),
            w.as_usize(),
            x.as_usize()))
    }
}

pub fn safe_efi_call5<U, V, W, X, Y, Z>(f:extern fn(U, V, W, X, Y) -> Z,
                              u:U, v:V, w:W, x:X, y:Y)
    -> Z
        where U: EfiParameter,
              V: EfiParameter,
              W: EfiParameter,
              X: EfiParameter,
              Y: EfiParameter,
              Z: EfiOutput,
{
    unsafe {
        EfiOutput::from_usize(efi_call5(
            mem::transmute(f),
            u.as_usize(),
            v.as_usize(),
            w.as_usize(),
            x.as_usize(),
            y.as_usize()))
    }
}

pub fn safe_reset_efi_call<U, V, W, X>(f:extern fn (U, V, W, X) -> !,
                                u:U, v:V, w:W, x:X) -> !
        where U: EfiParameter,
              V: EfiParameter,
              W: EfiParameter,
              X: EfiParameter,
{
    unsafe {
        efi_call4(
            mem::transmute(f),
            u.as_usize(),
            v.as_usize(),
            w.as_usize(),
            x.as_usize());
        unreachable!()
    }
}
