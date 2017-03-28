#include <efi.h>
#include <efilib.h>
#include <efibind.h>

#include "gdb_stub.h"

extern int main(void);
extern void rust_main(EFI_HANDLE, EFI_SYSTEM_TABLE *);

EFI_STATUS
EFIAPI
efi_main (EFI_HANDLE ImageHandle, EFI_SYSTEM_TABLE *SystemTable)
{
    EFI_LOADED_IMAGE *loaded_image = NULL;
    EFI_STATUS status;

    // Initialize efilib with main arguments
    InitializeLib(ImageHandle, SystemTable);
    status = uefi_call_wrapper(SystemTable->BootServices->HandleProtocol,
                                3,
                                ImageHandle,
                                &LoadedImageProtocol,
                                (void **)&loaded_image);
    if (EFI_ERROR(status)) {
        Print(L"handleprotocol: %r\n", status);
    }

    // Wait for debugger (should be ifdeffed about being in debug mode
    gdb_stub();

    // Start rust code
    rust_main(ImageHandle, SystemTable);
    return EFI_SUCCESS;
}


