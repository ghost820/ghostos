#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel64::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

use kernel64::memory::{self, PhysicalFrameAllocator};
use kernel64::println;
use kernel64::task::executor::Executor;
use kernel64::task::{Task, task_keyboard};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    kernel64::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::get_offset_page_table(phys_mem_offset) };
    let mut frame_allocator = unsafe { PhysicalFrameAllocator::new(&boot_info.memory_map) };
    memory::init(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(task_keyboard::task()));
    executor.run();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel64::test_panic_handler(info)
}
