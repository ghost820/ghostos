#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[cfg(test)]
use bootloader::{BootInfo, entry_point};

pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod uart;
pub mod vga_text_buffer;

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();

    test_main();

    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[fail]\n");
    serial_println!("Error: {}\n", info);

    exit_qemu(QemuExitCode::Failure);

    loop {
        x86_64::instructions::hlt();
    }
}

pub fn init() {
    gdt::init();
    interrupts::init();

    x86_64::instructions::interrupts::enable();
}

// Exit codes shound not overlap with QEMU's own exit codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0xf4);
    unsafe {
        port.write(exit_code as u32);
    }
}

// &[&dyn Fn()]
pub fn test_runner(tests: &[&dyn TestCase]) {
    serial_println!("\nRunning {} tests", tests.len());
    for test in tests {
        test.run();
    }
    serial_println!();

    exit_qemu(QemuExitCode::Success);
}

pub trait TestCase {
    fn run(&self) -> ();
}

impl<T> TestCase for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}... ", core::any::type_name::<T>());

        self();

        serial_println!("[ok]");
    }
}
