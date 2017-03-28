

use ::api::types::FunctionPointer;

#[repr(C)]
pub struct SimpleTextOutputMode {
    max_mode: i32,
    mode: i32,
    attribute: i32,
    cursor_column: i32,
    cursor_row: i32,
    cursor_visible: bool,
}

#[repr(C)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct SimpleTextOutputProtocol {
    Reset: FunctionPointer,
    OutputString: FunctionPointer,
    TestString: FunctionPointer,
    QueryMode: FunctionPointer,
    SetMode: FunctionPointer,
    SetAttribute: FunctionPointer,
    ClearScreen: FunctionPointer,
    SetCursorPosition: FunctionPointer,
    EnableCursor: FunctionPointer,
    mode: *const SimpleTextOutputMode,
}
