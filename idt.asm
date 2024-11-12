section .asm

global IDTSetIDTR

IDTSetIDTR:
    push ebp
    mov ebp, esp

    mov eax, [ebp + 8]
    lidt [eax]

    pop ebp
    ret
