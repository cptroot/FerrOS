#include <efi.h>
#include <efilib.h>

void gdb_stub() {
#ifdef DEBUG_MODE
    Print(L"Waiting for GDB\n");

    int wait = 1;
    while (wait) {
        __asm__ __volatile__("pause");
    }

    Print(L"Linked with GDB\n");
#endif
}
