use core::arch::global_asm;
use core::mem::offset_of;

use x86_64::VirtAddr;
use x86_64::registers::rflags::RFlags;

use crate::gdt;

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct UserContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

impl UserContext {
    pub fn new(entry_point: VirtAddr, stack_pointer: VirtAddr) -> Self {
        const RFLAGS_RESERVED_BIT: u64 = 1 << 1;

        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rdi: 0,
            rsi: 0,
            rbp: 0,
            rdx: 0,
            rcx: 0,
            rbx: 0,
            rax: 0,
            instruction_pointer: entry_point.as_u64(),
            code_segment: u64::from(gdt::user_code_selector().0),
            flags: RFLAGS_RESERVED_BIT | RFlags::INTERRUPT_FLAG.bits(),
            stack_pointer: stack_pointer.as_u64(),
            stack_segment: u64::from(gdt::user_data_selector().0),
        }
    }
}

global_asm!(
    r#"
.global user_context_enter
.type user_context_enter, @function

user_context_enter:
    push qword ptr [rdi + {stack_segment}]
    push qword ptr [rdi + {stack_pointer}]
    push qword ptr [rdi + {flags}]
    push qword ptr [rdi + {code_segment}]
    push qword ptr [rdi + {instruction_pointer}]

    mov r15, qword ptr [rdi + {r15}]
    mov r14, qword ptr [rdi + {r14}]
    mov r13, qword ptr [rdi + {r13}]
    mov r12, qword ptr [rdi + {r12}]
    mov r11, qword ptr [rdi + {r11}]
    mov r10, qword ptr [rdi + {r10}]
    mov r9, qword ptr [rdi + {r9}]
    mov r8, qword ptr [rdi + {r8}]
    mov rsi, qword ptr [rdi + {rsi}]
    mov rbp, qword ptr [rdi + {rbp}]
    mov rdx, qword ptr [rdi + {rdx}]
    mov rcx, qword ptr [rdi + {rcx}]
    mov rbx, qword ptr [rdi + {rbx}]
    mov rax, qword ptr [rdi + {rax}]
    mov rdi, qword ptr [rdi + {rdi}]

    iretq

.size user_context_enter, .-user_context_enter
"#,
    r15 = const offset_of!(UserContext, r15),
    r14 = const offset_of!(UserContext, r14),
    r13 = const offset_of!(UserContext, r13),
    r12 = const offset_of!(UserContext, r12),
    r11 = const offset_of!(UserContext, r11),
    r10 = const offset_of!(UserContext, r10),
    r9 = const offset_of!(UserContext, r9),
    r8 = const offset_of!(UserContext, r8),
    rdi = const offset_of!(UserContext, rdi),
    rsi = const offset_of!(UserContext, rsi),
    rbp = const offset_of!(UserContext, rbp),
    rdx = const offset_of!(UserContext, rdx),
    rcx = const offset_of!(UserContext, rcx),
    rbx = const offset_of!(UserContext, rbx),
    rax = const offset_of!(UserContext, rax),
    instruction_pointer = const offset_of!(UserContext, instruction_pointer),
    code_segment = const offset_of!(UserContext, code_segment),
    flags = const offset_of!(UserContext, flags),
    stack_pointer = const offset_of!(UserContext, stack_pointer),
    stack_segment = const offset_of!(UserContext, stack_segment),
);

unsafe extern "C" {
    fn user_context_enter(context: *const UserContext) -> !;
}

pub(crate) fn enter(context: &UserContext) -> ! {
    unsafe {
        user_context_enter(context);
    }
}
