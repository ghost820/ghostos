#![no_std]
#![no_main]

use core::panic::PanicInfo;

// cargo.exe bootimage && qemu-system-x86_64 -drive format=raw,file=target/target/debug/bootimage-kernel64.bin

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
