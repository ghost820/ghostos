#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel64::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use core::panic::PanicInfo;

use bootloader_api::{BootInfo, BootloaderConfig, entry_point};

use kernel64::interrupts;
use kernel64::memory::{HEAP_SIZE, KERNEL_SPACE_ADDR};

const BOOTLOADER_CONFIG: BootloaderConfig = {
    use bootloader_api::config::Mapping;

    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.mappings.dynamic_range_start = Some(KERNEL_SPACE_ADDR as u64);
    config.mappings.dynamic_range_end = Some(0xffff_bfff_ffff_f000);
    config
};

entry_point!(main, config = &BOOTLOADER_CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    use kernel64::memory;
    use x86_64::VirtAddr;

    kernel64::init();

    let phys_mem_offset = VirtAddr::new(
        boot_info
            .physical_memory_offset
            .into_option()
            .expect("physical memory mapping is unavailable"),
    );
    memory::init(&boot_info.memory_regions, phys_mem_offset).expect("memory initialization failed");

    interrupts::enable();

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
