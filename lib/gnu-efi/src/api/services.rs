use super::types;
use super::super::bind;

use ::mem::c_void;

use super::types::{FunctionPointer, TableHeader};

#[repr(C)]
pub struct BootServices {
    hdr:					TableHeader,

    //
    // Task priority functions
    //

    RaiseTPL:                   FunctionPointer,
    RestoreTPL:                 FunctionPointer,

    //
    // Memory functions
    //

    AllocatePages:  extern fn(allocate_type: def::AllocateType, memory_type: def::MemoryType, pages: usize, buffer: *const *mut u8) -> def::Status,
    FreePages:                  FunctionPointer,
    GetMemoryMap:   extern fn(memory_map_size:&mut usize, memory_map:*const def::MemoryDescriptor, map_key:&mut usize, descriptor_size:&mut usize, descriptor_version:&mut u32) -> def::Status,
    AllocatePool:   extern fn(pool_type: def::MemoryType, size: usize, buffer: *const *mut u8) -> def::Status,
    FreePool:       extern fn(buffer: *const c_void) -> def::Status,

    //
    // Event & timer functions
    //

    CreateEvent:                FunctionPointer,
    SetTimer:                   FunctionPointer,
    WaitForEvent:                   FunctionPointer,
    SignalEvent:                FunctionPointer,
    CloseEvent:                 FunctionPointer,
    CheckEvent:                 FunctionPointer,

    //
    // Protocol handler functions
    //

    InstallProtocolInterface:                   FunctionPointer,
    ReinstallProtocolInterface:                 FunctionPointer,
    UninstallProtocolInterface:                 FunctionPointer,
    HandleProtocol:             extern fn(image_handle: *const c_void, guid: *const types::Guid, interface: *const *mut c_void)->def::Status,
    PCHandleProtocol:                   FunctionPointer,
    RegisterProtocolNotify:                 FunctionPointer,
    LocateHandle:               extern fn(search_type: ::api::types::LocateSearchType, protocol: *const types::Guid, search_key: *const c_void, buffer_size: *mut usize, buffer: *mut def::Handle) -> def::Status,
    LocateDevicePath:                   FunctionPointer,
    InstallConfigurationTable:                  FunctionPointer,

    //
    // Image functions
    //

    LoadImage:                  FunctionPointer,
    StartImage:                 FunctionPointer,
    Exit:                   FunctionPointer,
    UnloadImage:                FunctionPointer,
    ExitBootServices:   extern fn(image_handle:*const c_void, map_key:usize)->def::Status,

    //
    // Misc functions
    //

    GetNextMonotonicCount:                  FunctionPointer,
    Stall:                  FunctionPointer,
    SetWatchdogTimer:                   FunctionPointer,

    //
    // DriverSupport Services
    //

    ConnectController:                  FunctionPointer,
    DisconnectController:                   FunctionPointer,

    //
    // Open and Close Protocol Services
    //
    OpenProtocol:                   FunctionPointer,
    CloseProtocol:                  FunctionPointer,
    OpenProtocolInformation:                FunctionPointer,

    //
    // Library Services
    //
    ProtocolsPerHandle:                 FunctionPointer,
    LocateHandleBuffer:                 extern fn(search_type: ::api::types::LocateSearchType, protocol: *const types::Guid, search_key: *const c_void, buffer_size: &mut usize, buffer: &mut *const def::Handle) -> def::Status,
    LocateProtocol:                 FunctionPointer,
    InstallMultipleProtocolInterfaces:                  FunctionPointer,
    UninstallMultipleProtocolInterfaces:                FunctionPointer,

    //
    // 32-bit CRC Services
    //
    CalculateCrc32:                 FunctionPointer,

    //
    // Misc Services
    //
    CopyMem:                FunctionPointer,
    SetMem:                 FunctionPointer,
    CreateEventEx:                  FunctionPointer,
}

#[repr(C)]
pub struct RuntimeServices {
    hdr:					    TableHeader,

    //
    // Time services
    //
    GetTime:                    FunctionPointer,
    SetTime:                    FunctionPointer,
    GetWakeupTime:              FunctionPointer,
    SetWakeupTime:              FunctionPointer,

    //
    // Virtual memory services
    //
    SetVirtualAddressMap:       FunctionPointer,
    ConvertPointer:             FunctionPointer,

    //
    // Variable serviers
    //
    GetVariable:                FunctionPointer,
    GetNextVariableName:        FunctionPointer,
    SetVariable:                FunctionPointer,

    //
    // Misc
    //
    GetNextHighMonotonicCount:  FunctionPointer,
    ResetSystem:                extern fn(reset_type:types::ResetType, reset_status:def::Status, data_size:usize, reset_data:*const c_void) -> !,
}

use core::mem;
use def;
use ::api::protocol::Protocol;

impl BootServices {
    pub fn get_memory_map(&self, buffer:&mut [u8]) -> (def::MemoryDescriptors, usize) {
        let mut memory_map_size = buffer.len();
        unsafe {
            let mut map_key = mem::uninitialized();
            let mut descriptor_size = mem::uninitialized();
            let mut descriptor_version = mem::uninitialized();

            let status = bind::safe_efi_call5(
                self.GetMemoryMap,
                    &mut memory_map_size,
                    buffer.as_mut_ptr() as *mut def::MemoryDescriptor,
                    &mut map_key,
                    &mut descriptor_size,
                    &mut descriptor_version);

            if status != def::Status::Success {
                panic!("Unable to get memory map: {:?}", status);
            }

            let memory_map = def::MemoryDescriptors::new(
                buffer.as_ptr() as *const def::MemoryDescriptor,
                memory_map_size / descriptor_size,
                descriptor_size);

            (memory_map, map_key)
        }
    }

    pub fn exit_boot_services(&self, image_handle: &def::Handle, map_key: usize) {
        let status = bind::safe_efi_call2(
            self.ExitBootServices,
            image_handle.handle,
            map_key);
        if status != def::Status::Success {
            panic!("Unable to exit boot services: {:?}", status);
        }
    }

    pub fn retrieve_handles_with_protocol<T: Protocol>(&self) -> Result<&[def::Handle], def::Status> {
        let mut buffer_size: usize = 0;
        let mut buffer: *const def::Handle = 0 as *mut def::Handle;
        let status = bind::safe_efi_call5(
            self.LocateHandleBuffer,
            ::api::types::LocateSearchType::ByProtocol,
            &T::get_guid(),
            0 as *mut c_void,
            &mut buffer_size,
            &mut buffer);

        unsafe {
            if status == def::Status::Success {
                Ok(::core::slice::from_raw_parts(buffer, buffer_size))
            } else {
                Err(status)
            }
        }
    }

    pub fn retrieve_protocol_from_handle<T: Protocol>(&self, handle: &def::Handle) -> Result<&mut T, def::Status> {
        let pointer: *mut T = 0 as *mut T;
        let status = bind::safe_efi_call3(
            self.HandleProtocol,
            handle.handle,
            &T::get_guid(),
            (&pointer as *const *mut T) as *const *mut c_void);

        if status == def::Status::Success {
            unsafe {
                Ok(::core::mem::transmute(pointer))
            }
        } else {
           Err(status) 
        }
    }

    pub fn allocate_pages(&self, pages: usize) -> Result<types::EfiBuffer, def::Status> {
            let pointer: *mut u8 = 0 as *mut u8;
            let status = bind::safe_efi_call4(
                self.AllocatePages,
                def::AllocateType::AllocateAnyPages,
                def::MemoryType::LoaderData,
                pages,
                &pointer as *const *mut u8);

        unsafe {
            let result = types::EfiBuffer::new(pointer, pages * 0x1000);

            if status == def::Status::Success {
                Ok(result)
            } else {
                Err(status)
            }
        }
    }

    pub fn allocate_pool(&self, size: usize) -> Result<types::EfiBuffer, def::Status> {
            let pointer: *mut u8 = 0 as *mut u8;
            let status = bind::safe_efi_call3(
                self.AllocatePool,
                def::MemoryType::LoaderData,
                size,
                &pointer as *const *mut u8);

        unsafe {
            let result = types::EfiBuffer::new(pointer, size);

            if status == def::Status::Success {
                Ok(result)
            } else {
                Err(status)
            }
        }
    }

    pub fn free_pool(&self, buffer:&types::EfiBuffer) -> Result<(), def::Status> {
        let status = bind::safe_efi_call1(
            self.FreePool,
            *buffer.get_pointer() as *const c_void);

        if status == def::Status::Success {
            Ok(())
        } else {
            Err(status)
        }
    }
}

impl RuntimeServices {
    pub fn reset_system(&self, reset_type:types::ResetType, reset_status:def::Status, data_size:usize, reset_data:*const c_void) -> ! {
        bind::safe_reset_efi_call(
            self.ResetSystem,
            reset_type,
            reset_status,
            data_size,
            reset_data);
    }
}
