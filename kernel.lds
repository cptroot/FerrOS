OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH(i386:x86-64)
ENTRY(kernel_entry)			/* Kernel starts at "start" symbol. */
SECTIONS
{
    /* Specify the location of the trampoline */
    _trampoline = 0x2000 ;
    . = _trampoline ;
    .trampoline . : { *(.trampoline) }

    ASSERT(. - _trampoline < 4K, "Trampoline is too big.")

    /* Specify the kernel base address. */
    _kernel_start = 0x8000000000 ;
    . = _kernel_start ;

    /* Kernel starts with code, followed by read-only data and writable data. */

    /* "kernel_ap_entry" = 0x8000010000 ; */
    .text : ALIGN(0x1000) { *(.text) *(.text.*) }
    .rodata : ALIGN(0x1000) { *(.rodata) *(.rodata.*) }
    .data : ALIGN(0x1000) { *(.data) *(.data.*) }
    .bss : ALIGN(0x1000) { *(.bss) *(.bss.*) }
}
