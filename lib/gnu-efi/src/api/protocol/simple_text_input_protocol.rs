
use ::api::types::FunctionPointer;

#[repr(C)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct SimpleTextInputProtocol {
    Reset: FunctionPointer,
    ReadKeyStroke: FunctionPointer,
    WaitForKey: ::api::types::Event,
}
