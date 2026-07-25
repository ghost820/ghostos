use core::arch::global_asm;

use crate::interrupts;
use crate::memory::kernel_stack_top;
use crate::syscall::{self, KernelSyscall};
use crate::userspace::context::{self, UserContext};
use crate::userspace::scheduler;

global_asm!(
    r#"
.global kernel_loop_enter
.type kernel_loop_enter, @function

kernel_loop_enter:
    mov r12, rdi

    call kernel_loop_stack_top

    mov rsp, rax

    mov rdi, r12
    call kernel_loop_entry
    ud2

.size kernel_loop_enter, .-kernel_loop_enter
"#
);

#[unsafe(no_mangle)]
extern "C" fn kernel_loop_stack_top() -> u64 {
    kernel_stack_top().as_u64()
}

#[unsafe(no_mangle)]
extern "C" fn kernel_loop_entry(context: *const UserContext) -> ! {
    let mut context = unsafe { &*context }.clone();

    let syscall = syscall::prepare_kernel_syscall(&mut context);

    scheduler::capture_running(&context);

    match syscall {
        KernelSyscall::Yield => scheduler::yield_current(),
        KernelSyscall::SleepUntil(deadline) => scheduler::sleep_current(deadline),
    }

    run();
}

pub fn run() -> ! {
    interrupts::disable();

    loop {
        if let Some(context) = scheduler::prepare_next() {
            context::enter(&context);
        }

        interrupts::enable_and_hlt();
        interrupts::disable();
    }
}
