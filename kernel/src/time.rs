use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;

use crate::io::{self, PortAddress, ReadWrite, WriteOnly};

pub const TIMER_FREQUENCY_HZ: u64 = 1_000;

const PIT_INPUT_FREQUENCY_HZ: u64 = 1_193_182;
const PIT_RELOAD_VALUE: u16 = (PIT_INPUT_FREQUENCY_HZ / TIMER_FREQUENCY_HZ) as u16;

const PIT_CHANNEL_0: PortAddress<u8, ReadWrite> = unsafe { PortAddress::new(0x40) };
const PIT_COMMAND: PortAddress<u8, WriteOnly> = unsafe { PortAddress::new(0x43) };

static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    const CHANNEL_0: u8 = 0b00 << 6;
    const LOW_THEN_HIGH_BYTE: u8 = 0b11 << 4;
    const RATE_GENERATOR: u8 = 0b010 << 1;
    const BINARY: u8 = 0;

    io::write(
        PIT_COMMAND,
        CHANNEL_0 | LOW_THEN_HIGH_BYTE | RATE_GENERATOR | BINARY,
    );
    io::write(PIT_CHANNEL_0, PIT_RELOAD_VALUE as u8);
    io::write(PIT_CHANNEL_0, (PIT_RELOAD_VALUE >> 8) as u8);
}

pub fn now() -> Duration {
    let ticks = u128::from(TICKS.load(Ordering::Relaxed));
    let pit_cycles = ticks * u128::from(PIT_RELOAD_VALUE);
    let frequency = u128::from(PIT_INPUT_FREQUENCY_HZ);

    let seconds = pit_cycles / frequency;
    let nanoseconds = pit_cycles % frequency * 1_000_000_000 / frequency;

    Duration::new(seconds as u64, nanoseconds as u32)
}

pub(crate) fn interrupt() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}
