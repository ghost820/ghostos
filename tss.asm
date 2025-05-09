section .asm

global TSSLoad

TSSLoad:
    push ebp
    mov ebp, esp

    mov ax, [ebp + 8]
    ltr ax

    pop ebp
    ret
