use lazy_static::lazy_static;
use spin::Mutex;

use pic8259::ChainedPics;
use x86_64::PrivilegeLevel;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::drivers::ps2;
use crate::println;
use crate::syscall;
use crate::time;
use ghostos_syscall::SYSCALL_INTERRUPT_VECTOR;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(0);
        }
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler);
        idt[32].set_handler_fn(timer_interrupt_handler);
        idt[33].set_handler_fn(keyboard_interrupt_handler);
        idt[39].set_handler_fn(irq7_interrupt_handler);
        idt[44].set_handler_fn(mouse_interrupt_handler);
        idt[47].set_handler_fn(irq15_interrupt_handler);
        unsafe {
            idt[SYSCALL_INTERRUPT_VECTOR]
                .set_handler_addr(syscall::interrupt_entry())
                .set_privilege_level(PrivilegeLevel::Ring3);
        }
        idt
    };
}

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(32, 40) });

pub fn init() {
    IDT.load();
}

pub fn init_pics() {
    unsafe {
        PICS.lock().initialize();
    }
}

pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

pub fn enable_and_hlt() {
    x86_64::instructions::interrupts::enable_and_hlt();
}

pub fn disable() {
    x86_64::instructions::interrupts::disable();
}

pub fn without_interrupts<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    x86_64::instructions::interrupts::without_interrupts(f)
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    // println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);

    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    panic!("invalid opcode\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\nError Code: {:#x}\n{:#?}",
        error_code, stack_frame
    );
}

//
// Using print! in both the main code and interrupt handlers can cause deadlocks.
// Heap allocation can cause deadlocks (lazy_static!).
//

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    time::interrupt();

    unsafe {
        PICS.lock().notify_end_of_interrupt(32);
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    ps2::keyboard::handle_scancode(ps2::controller::read_data_nowait());

    unsafe {
        PICS.lock().notify_end_of_interrupt(33);
    }
}

extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    ps2::mouse::handle_byte(ps2::controller::read_data_nowait());

    unsafe {
        PICS.lock().notify_end_of_interrupt(44);
    }
}

use x86_64::instructions::port::Port;

const MASTER_PIC_COMMAND_PORT: u16 = 0x20;
const SLAVE_PIC_COMMAND_PORT: u16 = 0xA0;

fn irq7_in_service(command_port: u16) -> bool {
    const READ_ISR_COMMAND: u8 = 0x0B;
    const IRQ7_IN_SERVICE: u8 = 1 << 7;

    unsafe {
        let mut port = Port::<u8>::new(command_port);

        port.write(READ_ISR_COMMAND);
        port.read() & IRQ7_IN_SERVICE != 0
    }
}

fn notify_master_end_of_interrupt() {
    unsafe {
        Port::<u8>::new(MASTER_PIC_COMMAND_PORT).write(0x20);
    }
}

extern "x86-interrupt" fn irq7_interrupt_handler(_stack_frame: InterruptStackFrame) {
    if irq7_in_service(MASTER_PIC_COMMAND_PORT) {
        unsafe {
            PICS.lock().notify_end_of_interrupt(39);
        }
    }
}

extern "x86-interrupt" fn irq15_interrupt_handler(_stack_frame: InterruptStackFrame) {
    if irq7_in_service(SLAVE_PIC_COMMAND_PORT) {
        unsafe {
            PICS.lock().notify_end_of_interrupt(47);
        }
    } else {
        notify_master_end_of_interrupt();
    }
}

#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}
