#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel64::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use core::panic::PanicInfo;

use bootloader::{BootInfo, entry_point};

use kernel64::memory::HEAP_SIZE;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use kernel64::memory::{self, PhysicalFrameAllocator};
    use x86_64::VirtAddr;

    kernel64::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::get_offset_page_table(phys_mem_offset) };
    let mut frame_allocator = unsafe { PhysicalFrameAllocator::new(&boot_info.memory_map) };
    memory::init(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    test_main();

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel64::test_panic_handler(info)
}

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn freed_memory_reuse() {
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}
