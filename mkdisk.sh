dd if=/dev/zero of=/tmp/uefi.img bs=512 count=93750
parted /tmp/uefi.img -s -a minimal mklabel gpt
parted /tmp/uefi.img -s -a minimal mkpart EFI FAT16 2048s 93716s
parted /tmp/uefi.img -s -a minimal toggle 1 boot
dd if=/dev/zero of=/tmp/part.img bs=512 count=91669
mformat -i /tmp/part.img -h 32 -t 32 -n 64 -c 1
mmd -i /tmp/part.img ::EFI
mmd -i /tmp/part.img ::EFI/BOOT
mmd -i /tmp/part.img ::EFI/OS
mcopy -i /tmp/part.img $LOADER_EFI ::EFI/BOOT
mcopy -i /tmp/part.img $KERNEL_EFI ::EFI/OS
dd if=/tmp/part.img of=/tmp/uefi.img bs=512 count=91669 seek=2048 conv=notrunc
mv /tmp/uefi.img $UEFI_IMG
