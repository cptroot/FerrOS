    .section .trampoline
    .code16

.func trampoline
.globl trampoline
trampoline:
    movw $0x3000, %bx
    movw $0x20, (%bx)

    movw $4, %ax
    add $0x4, %bx
    movw %ax, (%bx)

    movl $0x3FF0, %esp

check_a20:
 
    xor %ax, %ax ; ax = 0
    mov %ax, %dx
    mov %ax, %cx
    not %cx ; ax = 0xFFFF
 
    mov %bx, (%bx)
    push %ebx
 
    mov $0x0500, %di
    mov $0x0510, %si
 
    mov %dx, %es
    movb %es:(%di), %al
    push %ax
 
    mov %cx, %es
    movb %es:(%si), %al
    push %ax
 
    mov %dx, %es
    movb $0x00, %es:(%di)

    mov %cx, %es
    movb $0xFF, %es:(%si)
 
    mov %dx, %es
    cmpb $0xFF, %es:(%di)
 
    pop %ax
    mov %cx, %es
    movb %al, %es:(%si)
 
    pop %ax
    mov %dx, %es
    movb %al, %es:(%di)
 
    je enable_a20_line
 
    mov $0x21, %ax

    add $0x4, %bx
    movw %ax, (%bx)
    push %eax

enable_protected_mode:

# enable gdt

# enable paging

# disable old paging first (might not be required)
#    mov %cr0, %eax
#    add $0x4, %bx
#    mov %eax, (%bx)
#    hlt
#    and $0x7FFFFFFF, %eax
#    mov %eax, %cr0

# set cr3 (pml4)
    mov $0x3100, %bx
    mov (%bx), %eax
    mov %eax, %cr3

    mov %cr3, %eax
    add $0x8, %bx
    mov %eax, (%bx)
    push %eax

# enable PAE mode
    mov %cr4, %eax          # Set the A-register to control register 4.
    or $1 << 5, %eax        # Set the PAE-bit, which is the 6th bit (bit 5).
    mov %eax, %cr4          # Set control register 4 to the A-register.

# enable long mode paging
    mov $0xC0000080, %ecx   # Set the C-register to 0xC0000080, which is the EFER MSR.
    rdmsr                   # Read from the model-specific register.
    or $1 << 8, %eax        # Set the LM-bit which is the 9th bit (bit 8).
    wrmsr                   # Write to the model-specific register.

# enable paging
    mov %cr0, %eax          # Set the A-register to control register 0.
    or $0x80000001, %eax       # Set the PG-bit, which is the 32nd bit (bit 31).
    mov %eax, %cr0          # Set control register 0 to the A-register.

# enable 64 bit mode
    cli
    mov Pointer, %eax
    lgdtl Pointer

    add $0x4, %bx
    mov %eax, (%bx)
    push %eax

    add $0x4, %bx
    movl $0x1, (%bx)
    pushl $0x1

    ljmp $0x10, $Realm64

.code64
testing:
    call Realm64

.code16

enable_a20_line:
    add $0x4, %bx
    movw %ax, (%bx)
 
    hlt


.endfunc



.align 0x10
GDT64:                           # Global Descriptor Table (64-bit).
    GDTNull:                       # The null descriptor.
    .quad 0
    GDTUnused:                       # The null descriptor.
    .quad 0
    GDTCode:                       # The code descriptor.
    .word 0x0000                    # Limit (low).
    .word 0                         # Base (low).
    .byte 0                         # Base (middle)
    .byte 0x9A                      # Access.
    .byte 0x20                      # Granularity.
    .byte 0                         # Base (high).
    GDTData:                       # The data descriptor.
    .word 0x0000                    # Limit (low).
    .word 0                         # Base (low).
    .byte 0                         # Base (middle)
    .byte 0x92                      # Access.
    .byte 0x80                      # Granularity.
    .byte 0                         # Base (high).
#GDT64:                           # Global Descriptor Table (64-bit).
#    .equ .null, . - GDT64         # The null descriptor.
#    .quad 0
#    .equ .code, . - GDT64         # The code descriptor.
#    .quad (1<<43) | (1<<44) | (1<<47) | (1<<53)
    Pointer:                    # The GDT-pointer.
    .word . - GDT64 - 1             # Limit.
    .quad GDT64                     # Base.

    .code64
Realm64:
# jump to rust
    sub $0x4, %esp
    movq $0xFFFFFFFFFFFFFFFF, %rbx
    push %rbx

    mov $0x0, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %fs
    mov %ax, %gs
    mov %ax, %ss

    mov %cr0, %rax
    or $1 << 1, %eax
    mov %rax, %cr0

    mov %cr4, %rax
    or $1 << 9, %eax
    or $1 << 10, %eax
    mov %rax, %cr4


    mov $0x3200, %rax
    mov (%rax), %rax

    callq *%rax
    hlt
 
