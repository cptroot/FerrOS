
use ::core::marker::PhantomData;

//pub use File;


pub struct File<'a> {
    buffer: *mut u8,
    size: usize,
    phantom_data: PhantomData<&'a u8>,
}


impl<'a> File<'a> {
    pub fn from_buffer<'b>(buffer: &'b mut [u8]) -> File<'b> {
        File {
            buffer: buffer.as_mut_ptr(),
            size: buffer.len(),
            phantom_data: PhantomData,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn file_header(&self) -> &FileHeader {
        unsafe {
            ::core::mem::transmute(self.buffer)
        }
    }

    pub fn section_headers(&'a self) -> SectionHeaders<'a> {
        let pointer = unsafe { self.buffer.offset(self.file_header().section_header_offset) as *const SectionHeader };
        SectionHeaders {
            start: pointer,
            current: 1,
            file: self,
        }
    }
}

#[repr(packed)]
#[derive(Debug)]
pub struct FileHeader {
    identifier: FileHeaderIdentifier,
    elf_type: ElfType,
    machine: InstructionSetArchitecture,
    version: u32,
    entry: usize,
    program_header_offset: isize,
    section_header_offset: isize,
    flags: u32,
    elf_header_size: u16,
    program_header_entry_size: u16,
    program_header_humber: u16,
    section_header_entry_size: u16,
    section_header_number: u16,
    section_header_string_index: u16,
}

impl FileHeader {
    pub fn entry_ptr(&self) -> *const u8 {
        self.entry as *const u8
    }
}

#[repr(packed)]
#[derive(Debug)]
pub struct FileHeaderIdentifier {
    magic_number: [u8; 4],
    class: u8,
    data: u8,
    version: u8,
    os_abi: u8,
    abi_version: u8,
    abi_padding: [u8; 7],
}

#[repr(u16)]
#[derive(Debug)]
pub enum ElfType {
    None = 0,
    Relocatable = 1,
    Executable = 2,
    Dynamic = 3,
    Core = 4,
    LOOS = 0xfe00,
    HIOS = 0xfeff,
    LOPROC = 0xff00,
    HIPROC = 0xffff,
}

#[repr(u16)]
#[derive(Debug)]
enum InstructionSetArchitecture {
    NoSpecific = 0,
    SPAC = 0x02,
    x86 = 0x03,
    IA_64 = 0x32,
    x86_64 = 0x3e,
}

pub struct SectionHeaders<'a> {
    start: *const SectionHeader,
    current: usize,
    file: &'a File<'a>,
}

impl<'a> Iterator for SectionHeaders<'a> {
    type Item = &'a SectionHeader;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            _ if self.current < self.file.file_header().section_header_number as usize => {
                let pointer = self.start as *const u8;
                let result = unsafe { Some(::core::mem::transmute(pointer.offset(((self.file.file_header().section_header_entry_size as usize) * self.current) as isize))) };
                self.current += 1;
                result
            },
            _ => None,
        }

    }
}

#[repr(packed)]
#[derive(Debug)]
pub struct SectionHeader {
    name_index: u32,
    pub section_type: SectionType,
    flags: usize,
    pub virtual_address: *mut u8,
    file_offset: isize,
    pub size: usize,
    link: u32,
    info: u32,
    address_align: u64,
    entry_size: u64,
}

impl SectionHeader {
    pub fn offset_buffer(&self, file: &File) -> &[u8] {
        unsafe {
            ::core::slice::from_raw_parts(file.buffer.offset(self.file_offset), self.size)
        }
    }
    pub fn addr_ptr(&self) -> *mut u8 {
        self.virtual_address
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum SectionType {
    Null = 0,
    ProgramBits = 1,
    SymbolTable = 2,
    StringTable = 3,
    Rela = 4,
    SymbolHashTable = 5,
    DynamicLinkingTable = 6,
    Note = 7,
    NoBits = 8,
    Rel = 9,
    SectionHeaderLib = 10,
    DynamicLoaderSymbolTable = 11,
    LOOS = 0x60000000,
    HIOS = 0x6FFFFFFF,
    LOPROC = 0x70000000,
    HIPROC = 0x7FFFFFFF,
}
