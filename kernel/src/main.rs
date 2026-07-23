#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel64::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use bootloader_api::{BootInfo, BootloaderConfig, entry_point};
use x86_64::VirtAddr;

use kernel64::drivers;
use kernel64::interrupts;
use kernel64::memory::{self, PhysicalFrameAllocator};
use kernel64::task::Task;
use kernel64::task::executor::Executor;
#[allow(unused_imports)]
use kernel64::{critical, debug, error, info, println, warning};

const BOOTLOADER_CONFIG: BootloaderConfig = {
    use bootloader_api::config::Mapping;

    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel64::init();

    let phys_mem_offset = VirtAddr::new(
        boot_info
            .physical_memory_offset
            .into_option()
            .expect("physical memory mapping is unavailable"),
    );
    let mut mapper = unsafe { memory::get_offset_page_table(phys_mem_offset) };
    let mut frame_allocator = unsafe { PhysicalFrameAllocator::new(&boot_info.memory_regions) };
    memory::init(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    drivers::ps2::keyboard::init();

    if let Err(error) = drivers::ps2::mouse::init() {
        error!("Failed to initialize PS/2 mouse: {:?}", error);
    }

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(drivers::ps2::keyboard::task()));
    executor.spawn(Task::new(drivers::ps2::mouse::task()));

    interrupts::enable();

    info!("Kernel initialized, starting task executor...");

    executor.run();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // TODO: Deadlock here
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
