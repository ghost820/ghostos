#![no_std]

use core::arch::asm;
use core::time::Duration;

pub const SYSCALL_INTERRUPT_VECTOR: u8 = 0x80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum SyscallNumber {
    Exit = 0,
    Yield = 1,
    SleepUntil = 2,
    TimeNow = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownSyscallNumber(u64);

impl UnknownSyscallNumber {
    pub fn raw(self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for SyscallNumber {
    type Error = UnknownSyscallNumber;

    fn try_from(number: u64) -> Result<Self, Self::Error> {
        match number {
            0 => Ok(Self::Exit),
            1 => Ok(Self::Yield),
            2 => Ok(Self::SleepUntil),
            3 => Ok(Self::TimeNow),
            number => Err(UnknownSyscallNumber(number)),
        }
    }
}

pub fn exit(status: u64) -> ! {
    unsafe {
        asm!(
            "int {vector}",
            vector = const SYSCALL_INTERRUPT_VECTOR,
            in("rax") SyscallNumber::Exit as u64,
            in("rdi") status,
            options(noreturn),
        );
    }
}

pub fn yield_now() {
    unsafe {
        asm!(
            "int {vector}",
            vector = const SYSCALL_INTERRUPT_VECTOR,
            inout("rax") SyscallNumber::Yield as u64 => _,
        );
    }
}

pub fn sleep_until(deadline: Duration) {
    let deadline_nanoseconds = deadline.as_nanos().min(u64::MAX as u128) as u64;

    unsafe {
        asm!(
            "int {vector}",
            vector = const SYSCALL_INTERRUPT_VECTOR,
            inout("rax") SyscallNumber::SleepUntil as u64 => _,
            in("rdi") deadline_nanoseconds,
        );
    }
}

pub fn time_now() -> Duration {
    let nanoseconds;

    unsafe {
        asm!(
            "int {vector}",
            vector = const SYSCALL_INTERRUPT_VECTOR,
            inout("rax") SyscallNumber::TimeNow as u64 => nanoseconds,
        );
    }

    Duration::from_nanos(nanoseconds)
}
