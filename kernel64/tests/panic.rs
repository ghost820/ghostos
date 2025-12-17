#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel64::{QemuExitCode, exit_qemu, serial_print, serial_println};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test();

    serial_println!("[failed]");

    exit_qemu(QemuExitCode::Failure);

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");

    exit_qemu(QemuExitCode::Success);

    loop {}
}

fn test() {
    serial_print!("\npanic... ");
    assert_eq!(0, 1);
}
