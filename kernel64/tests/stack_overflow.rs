#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

use lazy_static::lazy_static;

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use kernel64::{QemuExitCode, exit_qemu, serial_print, serial_println};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(0);
        }
        idt
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    serial_print!("\nstack_overflow... ");

    kernel64::gdt::init();
    TEST_IDT.load();

    stack_overflow();

    panic!("[fail]");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel64::test_panic_handler(info)
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]\n");

    exit_qemu(QemuExitCode::Success);

    loop {
        x86_64::instructions::hlt();
    }
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();
}
