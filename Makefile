
TARGET = x86_64-unknown-pintos

XARGO_ARGS = --target=$(TARGET)

UEFI_IMG = target/debug/uefi.img
RELEASE_UEFI_IMG = target/release/uefi.img
LOADER_DEBUG_EFI = target/debug/debug.efi
LOADER_EFI = target/debug/BOOTX64.efi
KERNEL_EFI = target/debug/kernel.efi
RELEASE_LOADER_EFI = target/release/BOOTX64.efi
RELEASE_KERNEL_EFI = target/release/kernel.efi
KERNEL_LIB = kernel/target/$(TARGET)/debug/libkernel.a
#KERNEL_LIB = kernel/target/debug/libkernel.a
LOADER_LIB = loader/target/$(TARGET)/debug/libloader.a
KERNEL_SO = target/debug/kernel.so

export RUST_TARGET_PATH = $(CURDIR)

all: $(UEFI_IMG) $(LOADER_DEBUG_EFI)

check:
	cd loader; xargo check $(XARGO_ARGS)
	cd kernel; xargo check $(XARGO_ARGS)  

test: test_efi test_loader test_kernel

test_efi:
	cd lib/gnu-efi; cargo test

test_loader:
	cd loader; xargo test $(XARGO_ARGS)

test_kernel:
	cd kernel; xargo test $(XARGO_ARGS) 

target/debug/gdb_stub.o: src/gdb_stub.c
	$(CC) src/gdb_stub.c     				\
		-c                              \
		-g								\
		-fno-stack-protector            \
		-fpic                           \
		-fshort-wchar                   \
		-mno-red-zone                   \
		-I /usr/include/efi/			\
		-I /usr/include/efi/x86_64/ 	\
		-DEFI_FUNCTION_WRAPPER          \
		-DDEBUG_MODE  					\
		-o target/debug/gdb_stub.o

target/release/gdb_stub.o: src/gdb_stub.c
	$(CC) src/gdb_stub.c     				\
		-c                              \
		-g								\
		-fno-stack-protector            \
		-fpic                           \
		-fshort-wchar                   \
		-mno-red-zone                   \
		-I /usr/include/efi/			\
		-I /usr/include/efi/x86_64/ 	\
		-DEFI_FUNCTION_WRAPPER          \
		-o target/release/gdb_stub.o

target/debug/main.o: src/efi_main.c
	$(CC) src/efi_main.c                  \
		-c                              \
		-g								\
		-fno-stack-protector            \
		-fpic                           \
		-fshort-wchar                   \
		-mno-red-zone                   \
		-I /usr/include/efi/			\
		-I /usr/include/efi/x86_64/ 	\
		-DEFI_FUNCTION_WRAPPER          \
		-o target/debug/main.o

$(KERNEL_LIB): $(shell find kernel/src -type f) $(shell find lib/gnu-efi/src -type f)
	cd kernel; xargo build $(XARGO_ARGS) 

$(LOADER_LIB): $(shell find loader/src -type f) $(shell find lib/gnu-efi/src -type f)
	cd loader; xargo build $(XARGO_ARGS)

target/debug/main.so: target/debug/main.o target/debug/gdb_stub.o $(LOADER_LIB)
	ld target/debug/main.o target/debug/gdb_stub.o $(LOADER_LIB)		\
		/usr/lib/crt0-efi-x86_64.o 			\
		-nostdlib							\
		-znocombreloc						\
		-T elf_x86_64_efi.lds 		\
		-shared								\
		-Bsymbolic							\
		-L /usr/lib 						\
		-l:libgnuefi.a						\
		-l:libefi.a							\
		-o target/debug/main.so

target/release/main.so: target/debug/main.o target/release/gdb_stub.o $(LOADER_LIB)
	ld target/debug/main.o target/release/gdb_stub.o $(LOADER_LIB)		\
		/usr/lib/crt0-efi-x86_64.o 			\
		-nostdlib							\
		-znocombreloc						\
		-T elf_x86_64_efi.lds 		\
		-shared								\
		-Bsymbolic							\
		-L /usr/lib 						\
		-l:libgnuefi.a						\
		-l:libefi.a							\
		-o target/release/main.so

$(LOADER_EFI): target/debug/main.so
	objcopy -j .text				\
			-j .sdata 				\
			-j .data				\
			-j .dynamic 			\
			-j .rel 				\
			-j .rela 				\
			-j .reloc 				\
			--target=efi-app-x86_64 \
			target/debug/main.so 				\
			$(LOADER_EFI)

$(RELEASE_LOADER_EFI): target/release/main.so
	objcopy -j .text				\
			-j .sdata 				\
			-j .data				\
			-j .dynamic 			\
			-j .rel 				\
			-j .rela 				\
			-j .reloc 				\
			--target=efi-app-x86_64 \
			target/release/main.so 				\
			$(RELEASE_LOADER_EFI)

$(LOADER_DEBUG_EFI): target/debug/main.so
	objcopy -j .text				\
			-j .bss					\
			-j .sdata 				\
			-j .data				\
			-j .dynamic 			\
			-j .rel 				\
			-j .rela 				\
			-j .reloc 				\
			-j .debug_info			\
			-j .debug_abbrev		\
			-j .debug_loc			\
			-j .debug_ranges		\
			-j .debug_line			\
			-j .debug_macinfo		\
			-j .debug_str			\
			-j .debug_pubnames		\
			-j .debug_pubtypes		\
			--target=efi-app-x86_64 \
			target/debug/main.so 				\
			$(LOADER_DEBUG_EFI)

$(KERNEL_SO): $(KERNEL_LIB) kernel.lds
	ld $(KERNEL_LIB)		\
		-nostdlib							\
		-znocombreloc						\
		-zmax-page-size=0x1000				\
		-T kernel.lds						\
		-Bstatic 							\
		-L /usr/lib 						\
		-l:libgnuefi.a						\
		-l:libefi.a							\
		-o $(KERNEL_SO)

$(KERNEL_EFI): $(KERNEL_SO)
	objcopy -j .text				\
			-j .rodata 				\
			-j .sdata 				\
			-j .data				\
			-j .bss 				\
			-j .dynamic 			\
			-j .rel 				\
			-j .rela 				\
			-j .reloc 				\
			--target=elf64-x86-64 \
			$(KERNEL_SO) \
			$(KERNEL_EFI)

$(UEFI_IMG): $(LOADER_EFI) $(KERNEL_EFI)
	dd if=/dev/zero of=/tmp/uefi.img bs=512 count=93750
	parted /tmp/uefi.img -s -a minimal mklabel gpt
	parted /tmp/uefi.img -s -a minimal mkpart EFI FAT16 2048s 93716s
	parted /tmp/uefi.img -s -a minimal toggle 1 boot
	dd if=/dev/zero of=/tmp/part.img bs=512 count=91669
	mformat -i /tmp/part.img -h 32 -t 32 -n 64 -c 1
	mmd -i /tmp/part.img ::EFI
	mmd -i /tmp/part.img ::EFI/BOOT
	mmd -i /tmp/part.img ::EFI/OS
	mcopy -i /tmp/part.img $(LOADER_EFI) ::EFI/BOOT
	mcopy -i /tmp/part.img $(KERNEL_EFI) ::EFI/OS
	dd if=/tmp/part.img of=/tmp/uefi.img \
		bs=512 count=91669 seek=2048 conv=notrunc
	mv /tmp/uefi.img $(UEFI_IMG)

$(RELEASE_UEFI_IMG): $(RELEASE_KERNEL_EFI) $(RELEASE_LOADER_EFI)
	dd if=/dev/zero of=/tmp/uefi.img bs=512 count=93750
	parted /tmp/uefi.img -s -a minimal mklabel gpt
	parted /tmp/uefi.img -s -a minimal mkpart EFI FAT16 2048s 93716s
	parted /tmp/uefi.img -s -a minimal toggle 1 boot
	dd if=/dev/zero of=/tmp/part.img bs=512 count=91669
	mformat -i /tmp/part.img -h 32 -t 32 -n 64 -c 1
	mmd -i /tmp/part.img ::EFI
	mmd -i /tmp/part.img ::EFI/BOOT
	mmd -i /tmp/part.img ::EFI/OS
	mcopy -i /tmp/part.img $(RELEASE_LOADER_EFI) ::EFI/BOOT
	mcopy -i /tmp/part.img $(RELEASE_KERNEL_EFI) ::EFI/OS
	dd if=/tmp/part.img of=/tmp/uefi.img \
		bs=512 count=91669 seek=2048 conv=notrunc
	mv /tmp/uefi.img $(RELEASE_UEFI_IMG)


run: $(RELEASE_UEFI_IMG)
	qemu-system-x86_64 -cpu qemu64 -smp cores=2,threads=1,sockets=1 \
		-bios OVMF/OVMF.fd -drive file=$(RELEASE_UEFI_IMG),if=ide \
		-nographic -monitor null -serial stdio

debug: all
	qemu-system-x86_64 -cpu qemu64 -smp cores=2,threads=1,sockets=1 \
		-bios OVMF/OVMF.fd -drive file=$(UEFI_IMG),if=ide \
		-nographic -monitor null -serial stdio -s

clean:
	rm -f target/debug/*.o target/debug/main.so $(PROGRAM_EFI) $(UEFI_IMG) $(LOADER_EFI) $(LOADER_DEBUG_EFI) $(KERNEL_EFI) $(KERNEL_SO)
	rm -f target/release/gdb_stub.o target/release/main.so target/release/BOOTX64.efi
	cd kernel; xargo clean $(XARGO_ARGS)
	cd loader; xargo clean $(XARGO_ARGS)
	cd lib/gnu-efi; cargo clean

.PHONY: all clean run check test test_efi test_kernel
