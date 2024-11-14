section .asm

global IDTSetIDTR
global int20h
global int21h
global intdh

extern int20h_handler
extern int21h_handler
extern intdh_handler

IDTSetIDTR:
    push ebp
    mov ebp, esp

    mov eax, [ebp + 8]
    lidt [eax]

    pop ebp
    ret

int20h:
    cli
    pushad

    call int20h_handler

    popad
    sti
    iret

int21h:
    cli
    pushad

    call int21h_handler

    popad
    sti
    iret

intdh:
    cli
    pushad

    call intdh_handler

    popad
    sti
    iret
