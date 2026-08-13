#![no_std]

use core::arch::asm;
use core::time::Duration;

pub mod input;

pub const SYSCALL_INTERRUPT_VECTOR: u8 = 0x80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum SyscallNumber {
    Exit = 0,
    Yield = 1,
    SleepUntil = 2,
    TimeNow = 3,
    InputState = 4,
    DebugLogByte = 5,
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
            4 => Ok(Self::InputState),
            5 => Ok(Self::DebugLogByte),
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

pub fn sleep_until(deadline: Duration) -> bool {
    let deadline_nanoseconds = deadline.as_nanos().min(u64::MAX as u128) as u64;
    let mut slept = 0u64;

    unsafe {
        asm!(
            "int {vector}",
            vector = const SYSCALL_INTERRUPT_VECTOR,
            inout("rax") SyscallNumber::SleepUntil as u64 => slept,
            in("rdi") deadline_nanoseconds,
        );
    }

    0 != slept
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

use input::{InputState, KeyboardState, MouseState};

pub fn input_state() -> InputState {
    let keyboard_word0;
    let keyboard_word1;
    let mouse_word;

    unsafe {
        asm!(
            "int {vector}",
            vector = const SYSCALL_INTERRUPT_VECTOR,
            inout("rax") SyscallNumber::InputState as u64 => keyboard_word0,
            out("rdx") keyboard_word1,
            out("rcx") mouse_word,
        );
    }

    InputState::new(
        KeyboardState::from_words([keyboard_word0, keyboard_word1]),
        MouseState::from_word(mouse_word),
    )
}

pub fn debug_log_byte(byte: u8) {
    unsafe {
        asm!(
            "int {vector}",
            vector = const SYSCALL_INTERRUPT_VECTOR,
            inout("rax") SyscallNumber::DebugLogByte as u64 => _,
            in("rdi") u64::from(byte),
        );
    }
}
