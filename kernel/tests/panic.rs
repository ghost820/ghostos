#![no_std]
#![no_main]

use core::panic::PanicInfo;

use bootloader_api::{BootInfo, entry_point};

use kernel64::{QemuExitCode, exit_qemu, serial_print, serial_println};

entry_point!(main);

fn main(_boot_info: &'static mut BootInfo) -> ! {
    test();

    serial_println!("[fail]");

    exit_qemu(QemuExitCode::Failure);

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]\n");

    exit_qemu(QemuExitCode::Success);

    loop {
        x86_64::instructions::hlt();
    }
}

fn test() {
    serial_print!("\npanic... ");
    assert_eq!(0, 1);
}
