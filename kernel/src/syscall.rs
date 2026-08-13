use core::arch::global_asm;
use core::time::Duration;

use x86_64::VirtAddr;

use crate::drivers::ps2::{keyboard, mouse};
use crate::time;
use crate::userspace::context::UserContext;
use ghostos_syscall::{SyscallNumber, UnknownSyscallNumber};

pub enum KernelSyscall {
    Yield,
    SleepUntil(Duration),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
enum SyscallDisposition {
    ReturnToUser = 0,
    EnterKernel = 1,
}

global_asm!(
    r#"
.global syscall_interrupt_entry
.type syscall_interrupt_entry, @function

syscall_interrupt_entry:
    push rax
    push rbx
    push rcx
    push rdx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    cld

    mov rdi, rsp
    call syscall_interrupt_dispatch

    test rax, rax
    jnz .enter_kernel

    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rdx
    pop rcx
    pop rbx
    pop rax

    iretq

    .enter_kernel:
    mov rdi, rsp
    jmp kernel_loop_enter

    .size syscall_interrupt_entry, .-syscall_interrupt_entry
"#
);

unsafe extern "C" {
    fn syscall_interrupt_entry();
}

pub fn interrupt_entry() -> VirtAddr {
    VirtAddr::from_ptr(syscall_interrupt_entry as *const ())
}

#[unsafe(no_mangle)]
extern "C" fn syscall_interrupt_dispatch(frame: &mut UserContext) -> SyscallDisposition {
    assert_eq!(
        frame.code_segment & 0b11,
        3,
        "syscall did not originate in ring 3"
    );

    match SyscallNumber::try_from(frame.rax) {
        Ok(SyscallNumber::Exit) => exit(frame.rdi),
        Ok(SyscallNumber::Yield) => SyscallDisposition::EnterKernel,
        Ok(SyscallNumber::SleepUntil) => {
            let deadline = Duration::from_nanos(frame.rdi);

            if deadline <= time::now() {
                frame.rax = 0;
                SyscallDisposition::ReturnToUser
            } else {
                SyscallDisposition::EnterKernel
            }
        }
        Ok(SyscallNumber::TimeNow) => {
            frame.rax = time::now().as_nanos().min(u64::MAX as u128) as u64;
            SyscallDisposition::ReturnToUser
        }
        Ok(SyscallNumber::InputState) => {
            let [word0, word1] = keyboard::snapshot().words();

            frame.rax = word0;
            frame.rdx = word1;
            frame.rcx = mouse::snapshot().word();

            SyscallDisposition::ReturnToUser
        }
        Ok(SyscallNumber::DebugLogByte) => {
            crate::serial_print!("{}", frame.rdi as u8 as char);

            frame.rax = 0;
            SyscallDisposition::ReturnToUser
        }
        Err(number) => reject_unknown_syscall(number),
    }
}

pub(crate) fn prepare_kernel_syscall(context: &mut UserContext) -> KernelSyscall {
    match SyscallNumber::try_from(context.rax) {
        Ok(SyscallNumber::Yield) => {
            context.rax = 0;
            KernelSyscall::Yield
        }
        Ok(SyscallNumber::SleepUntil) => {
            let deadline = Duration::from_nanos(context.rdi);

            context.rax = 1;
            KernelSyscall::SleepUntil(deadline)
        }
        _ => panic!("invalid kernel syscall"),
    }
}

fn reject_unknown_syscall(number: UnknownSyscallNumber) -> ! {
    panic!("unknown syscall: {}", number.raw());
}

fn exit(_status: u64) -> ! {
    unimplemented!("exit syscall");
}
