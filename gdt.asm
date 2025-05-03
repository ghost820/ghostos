section .asm

global GDTLoad

GDTLoad:
    mov eax, [esp + 4]
    mov [GDT_ADDR + 2], eax

    mov ax, [esp + 8]
    mov [GDT_ADDR], ax

    lgdt [GDT_ADDR]

    ret

section .data

GDT_ADDR:
    dw 0x00
    dd 0x00
