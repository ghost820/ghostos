section .asm

global EnableInterrupts
global DisableInterrupts
global IDTSetIDTR
global int20h
global int21h
global int80h
global intdh

extern int20h_handler
extern int21h_handler
extern int80h_handler
extern intdh_handler

EnableInterrupts:
    sti
    ret

DisableInterrupts:
    cli
    ret

IDTSetIDTR:
    push ebp
    mov ebp, esp

    mov eax, [ebp + 8]
    lidt [eax]

    pop ebp
    ret

int20h:
    pushad

    call int20h_handler

    popad
    iret

int21h:
    pushad

    call int21h_handler

    popad
    iret

int80h:
    pushad

    push esp
    push eax
    call int80h_handler
    mov dword [int80h_ret], eax
    add esp, 8

    popad
    mov eax, [int80h_ret]
    iret

intdh:
    pushad

    call intdh_handler

    popad
    iret

section .data

int80h_ret: dd 0
