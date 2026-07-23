#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel64::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader_api::{BootInfo, entry_point};

use kernel64::drivers::ata::{AtaDevice, ChannelId, DeviceId, Lba48};

entry_point!(main);

fn main(_boot_info: &'static mut BootInfo) -> ! {
    kernel64::init();

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
fn detects_device() {
    AtaDevice::probe(ChannelId::Primary, DeviceId::Slave).unwrap();
}

#[test_case]
fn reads_zeroed_sector() {
    let device = AtaDevice::probe(ChannelId::Primary, DeviceId::Slave)
        .expect("failed to detect test device");

    let mut buffer = [0u16; 256];

    device
        .read_sectors(Lba48::new(0), &mut buffer)
        .expect("failed to read test sector");

    assert!(buffer.iter().all(|&word| word == 0));
}

#[test_case]
fn writes_and_reads_sector() {
    let mut device = AtaDevice::probe(ChannelId::Primary, DeviceId::Slave)
        .expect("failed to detect test device");

    let write_buffer = [0xA55Au16; 256];

    device
        .write_sectors(Lba48::new(1), &write_buffer)
        .expect("failed to write test sector");

    device.flush().expect("failed to flush test device");

    let mut read_buffer = [0u16; 256];

    device
        .read_sectors(Lba48::new(1), &mut read_buffer)
        .expect("failed to read test sector");

    assert_eq!(read_buffer, write_buffer);
}
