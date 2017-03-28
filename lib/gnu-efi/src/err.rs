
const STATUS_ERROR: usize = 0x8000000000000000;

/// A representation of all UEFI error values
/// All error status have the high bit set
#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Status {
    Success             = 0,
    LoadError           = STATUS_ERROR | 1,
    InvalidParameter    = STATUS_ERROR | 2,
    Unsupported         = STATUS_ERROR | 3,
    BadBufferSize       = STATUS_ERROR | 4,
    BufferTooSmall      = STATUS_ERROR | 5,
    NotReady            = STATUS_ERROR | 6,
    DeviceError         = STATUS_ERROR | 7,
    WriteProtected      = STATUS_ERROR | 8,
    OutOfResources      = STATUS_ERROR | 9,
    VolumeCorrupted     = STATUS_ERROR | 10,
    VolumeFull          = STATUS_ERROR | 11,
    NoMedia             = STATUS_ERROR | 12,
    MediaChanged        = STATUS_ERROR | 13,
    NotFound            = STATUS_ERROR | 14,
    AccessDenied        = STATUS_ERROR | 15,
    NoResponse          = STATUS_ERROR | 16,
    NoMapping           = STATUS_ERROR | 17,
    Timeout             = STATUS_ERROR | 18,
    NotStarted          = STATUS_ERROR | 19,
    AlreadyStarted      = STATUS_ERROR | 20,
    Aborted             = STATUS_ERROR | 21,
    ICMPError           = STATUS_ERROR | 22,
    TFTPError           = STATUS_ERROR | 23,
    ProtocolError       = STATUS_ERROR | 24,
    IncompatibleVersion = STATUS_ERROR | 25,
    SecurityViolation   = STATUS_ERROR | 26,
    CRCError            = STATUS_ERROR | 27,
    EndOfMedia          = STATUS_ERROR | 28,
    //LoadError           = STATUS_ERROR | 29,
    //LoadError           = STATUS_ERROR | 30,
    EndOfFile           = STATUS_ERROR | 31,
    InvalidLanguage     = STATUS_ERROR | 32,
    CompromisedData     = STATUS_ERROR | 33,
    WarnUnknownGlyph    = 1,
    WarnDeleteFailure   = 2,
    WarnWriteFailure    = 3,
    WarnBufferTooSmall  = 4,
}
