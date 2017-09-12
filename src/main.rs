extern crate clap;

use std::process::Command;
use std::path::Path;

use clap::{App, SubCommand};

fn main() {
    let matches = App::new("FerrOS")
        .version("0.1.0")
        .author("Evan Davis <edavis@caltech.edu>")
        .about("Launcher for FerrOS instructional operating system")
        .subcommand(SubCommand::with_name("run")
            .about("Launches FerrOS")
            .version("0.1"))
        .subcommand(SubCommand::with_name("build")
            .about("Builds FerrOS")
            .version("0.1"))
        .subcommand(SubCommand::with_name("clean")
            .about("Cleans FerrOS artifacts")
            .version("0.1"))
        .get_matches();

    match matches.subcommand_name() {
        Some("run") => {
            let build_success = build();
            if build_success {
                run(None);
            }
        },
        Some("build") => {
            build();
        },
        Some("clean") => {
            clean();
        },
        _ => {
        }
    }

}

fn clean() -> bool {

    let debug_path = Path::new("./target_ferros/debug");
    let rm = if debug_path.exists() && debug_path.is_dir() {
        Some(Command::new("rm")
            .arg("-f")
            .args(&["gdb_stub.o", "efi_main.o", "trampoline.o"])
            .args(&["loader.so", "kernel.so"])
            .args(&["BOOTX64.efi", "debug.efi", "kernel.efi"])
            .arg("uefi.img")
            .current_dir("./target_ferros/debug")
            .spawn()
            .expect("unable to start rm"))
    } else {
        None
    };

    let mut loader_clean = Command::new("xargo")
        .env("RUST_TARGET_PATH", std::env::current_dir().expect("can't retrieve the current directory"))
        .current_dir("./loader")
        .arg("clean")
        .spawn()
        .expect("unable to start xargo");

    let loader_success = loader_clean.wait().map(|status| status.success()).unwrap_or(false);

    let mut kernel_clean = Command::new("xargo")
        .env("RUST_TARGET_PATH", std::env::current_dir().expect("can't retrieve the current directory"))
        .current_dir("./kernel")
        .arg("clean")
        .spawn()
        .expect("unable to start xargo");


    let mut success = true;
    success &= rm.and_then(|mut rm| Some(rm.wait().map(|status| status.success()).unwrap_or(false))).unwrap_or(true);
    success &= loader_success;
    success &= kernel_clean.wait().map(|status| status.success()).unwrap_or(false);

    return success;
}

fn build() -> bool {
    println!("Building ferros");
    Command::new("mkdir")
        .arg("-p")
        .arg("./target_ferros/debug")
        .spawn()
        .expect("failed to start mkdir")
        .wait()
        .ok()
        .and_then(|status| if status.success() { Some(()) } else { None })
        .expect("Making directories failed");
    let c_flags = [
        "-c",
        "-g",
        "-fno-stack-protector",
        "-fpic",
        "-fshort-wchar",
        "-mno-red-zone",
        "-I", "/usr/include/efi/",
        "-I", "/usr/include/efi/x86_64/",
        "-DEFI_FUNCTION_WRAPPER",
        "-DDEBUG_MODE",
    ];
    //let debug = true;

    let mut gdb_stub = Command::new("gcc")
        .args(&c_flags)
        .args(&["-o", "target_ferros/debug/gdb_stub.o"])
        .arg("src/gdb_stub.c")
        .spawn()
        .expect("failed to run gcc");

    let mut efi_main = Command::new("gcc")
        .args(&c_flags)
        .args(&["-o", "target_ferros/debug/efi_main.o"])
        .arg("src/efi_main.c")
        .spawn()
        .expect("failed to run gcc");

    let mut trampoline = Command::new("as")
        .args(&["-o", "target_ferros/debug/trampoline.o"])
        .arg("src/trampoline.S")
        .spawn()
        .expect("failed to run as");

    let mut loader = Command::new("xargo")
        .env("RUST_TARGET_PATH", std::env::current_dir().expect("can't retrieve the current directory"))
        .current_dir("./loader")
        .arg("build")
        .args(&["--target", "x86_64-unknown-pintos"])
        .spawn()
        .expect("failed to run xargo");
    let loader_success = loader.wait().map(|status| status.success()).unwrap_or(false);

    let mut kernel = Command::new("xargo")
        .env("RUST_TARGET_PATH", std::env::current_dir().expect("can't retrieve the current directory"))
        .current_dir("./kernel")
        .arg("build")
        .args(&["--target", "x86_64-unknown-pintos"])
        .spawn()
        .expect("failed to run xargo");

    let mut success = true;
    success &= gdb_stub.wait().map(|status| status.success()).unwrap_or(false);
    success &= efi_main.wait().map(|status| status.success()).unwrap_or(false);
    success &= trampoline.wait().map(|status| status.success()).unwrap_or(false);

    success &= loader_success;
    success &= kernel.wait().map(|status| status.success()).unwrap_or(false);

    if success {
        let mut loader_so = Command::new("ld")
            .args(&["target_ferros/debug/efi_main.o", "target_ferros/debug/gdb_stub.o", "loader/target/x86_64-unknown-pintos/debug/libloader.a"])
            .arg("/usr/lib/crt0-efi-x86_64.o")
            .arg("-nostdlib")
            .arg("-znocombreloc")
            .args(&["-T", "elf_x86_64_efi.lds"])
            .arg("-shared")
            .arg("-Bsymbolic")
            .args(&["-L", "/usr/lib"])
            .arg("-l:libgnuefi.a")
            .arg("-l:libefi.a")
            .args(&["-o", "target_ferros/debug/loader.so"])
            .spawn()
            .expect("failed to run ld");

        let mut kernel_so = Command::new("ld")
            .args(&["target_ferros/debug/trampoline.o", "kernel/target/x86_64-unknown-pintos/debug/libkernel.a"])
            .arg("-nostdlib")
            .arg("-znocombreloc")
            .arg("-zmax-page-size=0x1000")
            .args(&["-T", "kernel.lds"])
            .arg("-Bstatic")
            .args(&["-L", "/usr/lib"])
            .arg("-l:libgnuefi.a")
            .arg("-l:libefi.a")
            .args(&["-o", "target_ferros/debug/kernel.so"])
            .spawn()
            .expect("failed to run ld");

        let mut success = true;

        success &= loader_so.wait().map(|status| status.success()).unwrap_or(false);
        success &= kernel_so.wait().map(|status| status.success()).unwrap_or(false);

        if success {
            let mut loader_efi = Command::new("objcopy")
                .args(&["-j", ".text"])
                .args(&["-j", ".sdata"])
                .args(&["-j", ".data"])
                .args(&["-j", ".dynamic"])
                .args(&["-j", ".rel"])
                .args(&["-j", ".rela"])
                .args(&["-j", ".reloc"])
                .arg("--target=efi-app-x86_64")
                .arg("target_ferros/debug/loader.so")
                .arg("target_ferros/debug/BOOTX64.efi")
                .spawn()
                .expect("failed to run objcopy");

            let mut loader_debug_efi = Command::new("objcopy")
                .args(&["-j", ".text"])
                .args(&["-j", ".bss"])
                .args(&["-j", ".sdata"])
                .args(&["-j", ".data"])
                .args(&["-j", ".dynamic"])
                .args(&["-j", ".rel"])
                .args(&["-j", ".rela"])
                .args(&["-j", ".reloc"])
                .args(&["-j", ".debug_info"])
                .args(&["-j", ".debug_abbrev"])
                .args(&["-j", ".debug_loc"])
                .args(&["-j", ".debug_ranges"])
                .args(&["-j", ".debug_line"])
                .args(&["-j", ".debug_macinfo"])
                .args(&["-j", ".debug_str"])
                .args(&["-j", ".debug_pubnames"])
                .args(&["-j", ".debug_pubtypes"])
                .arg("--target=elf64-x86-64")
                .arg("target_ferros/debug/loader.so")
                .arg("target_ferros/debug/debug.efi")
                .spawn()
                .expect("failed to run objcopy");

            let mut kernel_efi = Command::new("objcopy")
                .args(&["-j", ".text"])
                .args(&["-j", ".rodata"])
                .args(&["-j", ".sdata"])
                .args(&["-j", ".data"])
                .args(&["-j", ".bss"])
                .args(&["-j", ".dynamic"])
                .args(&["-j", ".rel"])
                .args(&["-j", ".rela"])
                .args(&["-j", ".reloc"])
                .args(&["-j", ".trampoline"])
                .arg("--target=elf64-x86-64")
                .arg("target_ferros/debug/kernel.so")
                .arg("target_ferros/debug/kernel.efi")
                .spawn()
                .expect("failed to run objcopy");

            let mut success = true;
            success &= loader_efi.wait().map(|status| status.success()).unwrap_or(false);
            success &= kernel_efi.wait().map(|status| status.success()).unwrap_or(false);
            success &= loader_debug_efi.wait().map(|status| status.success()).unwrap_or(false);

            if success {
                let mut mkdisk = Command::new("sh")
                    .arg("mkdisk.sh")
                    .env("LOADER_EFI", "target_ferros/debug/BOOTX64.efi")
                    .env("KERNEL_EFI", "target_ferros/debug/kernel.efi")
                    .env("UEFI_IMG", "target_ferros/debug/uefi.img")
                    .spawn()
                    .expect("failed to run mkdisk.sh");

                let success = mkdisk.wait().map(|status| status.success()).unwrap_or(false);

                return success;
            }
        }
    }

    return false;
}

fn run(uefi_img: Option<&str>) {
    use std::io::{Read, Write};
    let uefi_img = uefi_img.unwrap_or("target_ferros/debug/uefi.img");
    let mut qemu = Command::new("qemu-system-x86_64")
        .args(&["-cpu", "qemu64"])
        .args(&["-smp", "cores=2,threads=1,sockets=1"])
        .args(&["-bios", "OVMF/OVMF.fd"])
        .args(&["-drive", &format!("file={UEFI_IMG},if=none,id=disk", UEFI_IMG=uefi_img)])
        .args(&["-device", "ide-drive,drive=disk,bootindex=1"])
        .arg("-nographic")
        .args(&["-monitor", "null"])
        .args(&["-serial", "stdio"])
        .arg("-s")
        .args(&["-d", "cpu_reset"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to start qemu");

    // Filter out screen clears
    let mut input_buffer = [0u8; 100];
    let mut output_buffer = [0u8; 100];
    let mut start_of_line = true;
    let mut filter_line = false;

    { // Begin NLL for qemu_stdout
    let qemu_stdout = qemu.stdout.as_mut().expect("stdout didn't pipe correctly");

    while let Ok(bytes_read) = qemu_stdout.read(&mut input_buffer) {
        if bytes_read == 0 {
            break;
        } else {
            let mut out_i = 0;
            for i in 0..bytes_read {
                if start_of_line {
                    if input_buffer[i] == 0x1b {
                        filter_line = true;
                    } else {
                        filter_line = false;
                    }
                    start_of_line = false;
                }
                if input_buffer[i] == b'\n' {
                    start_of_line = true;
                }

                if !filter_line {
                    output_buffer[out_i] = input_buffer[i];
                    out_i += 1;
                }
            }
            std::io::stdout().write(&output_buffer[0..out_i]);
        }
    }
    } //End NLL for qemu_stdout

    qemu.wait().expect("failed to wait for qemu to finish");
}
