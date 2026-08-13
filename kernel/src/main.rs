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
use kernel64::kernel_loop;
use kernel64::memory::{self, KERNEL_SPACE_ADDR};
use kernel64::userspace::loader::ExecutableImage;
use kernel64::userspace::process::Process;
use kernel64::userspace::scheduler;
#[allow(unused_imports)]
use kernel64::{critical, debug, error, info, println, warning};

const BOOTLOADER_CONFIG: BootloaderConfig = {
    use bootloader_api::config::Mapping;

    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.mappings.dynamic_range_start = Some(KERNEL_SPACE_ADDR as u64);
    config.mappings.dynamic_range_end = Some(0xffff_bfff_ffff_f000);
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
    memory::init(&boot_info.memory_regions, phys_mem_offset).expect("memory initialization failed");

    let framebuffer = boot_info
        .framebuffer
        .as_ref()
        .expect("framebuffer is unavailable");

    info!("{:?}", framebuffer.info());

    // TODO: This error handling is not correct
    if let Err(error) = drivers::ps2::mouse::init() {
        error!("Failed to initialize PS/2 mouse: {:?}", error);
    }

    interrupts::init_pics();

    drivers::ps2::keyboard::init();

    match drivers::e1000::find() {
        Some(function_addr) => {
            info!("E1000 found at {:?}", function_addr);

            match drivers::e1000::E1000::init(function_addr) {
                Ok(e1000) => info!("E1000 initialized: {:?}", e1000),
                // TODO: Error handling
                Err(error) => error!("Failed to initialize E1000: {:?}", error),
            }
        }
        None => warning!("E1000 not found"),
    }

    #[cfg(test)]
    test_main();

    let game = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../userspace/target/x86_64-ghostos/debug/game"
    ));

    let executable = ExecutableImage::new(game).expect("failed to parse game executable");

    let process =
        Process::new(&executable, 1_000_000, framebuffer).expect("failed to create game process");

    info!("Kernel initialized, starting main loop...");

    scheduler::add(process).expect("process limit reached");

    kernel_loop::run();
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
