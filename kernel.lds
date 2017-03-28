OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH(i386:x86-64)
ENTRY(kernel_entry)			/* Kernel starts at "start" symbol. */
SECTIONS
{
    /* Specify the kernel base address. */
    _start = 0x8000000000;
    . = _start;

    /* Kernel starts with code, followed by read-only data and writable data. */
    .text : ALIGN(0x1000) { *(.text) *(.text.*) } = 0x90
    .rodata : ALIGN(0x1000) { *(.rodata) *(.rodata.*) 
        . = ALIGN(0x1000); 
        _end_kernel_text = .; }
    .data : ALIGN(0x1000) { *(.data) *(.data.*) }

    /* BSS (zero-initialized data) is after everything else. */
    _start_bss = .;
    .bss : ALIGN(0x1000) { *(.bss) *(.bss.*) }
    _end_bss = .;

    _end = .;

    /*ASSERT(_end - _start <= 512K, "Kernel image is too big.")*/
}
