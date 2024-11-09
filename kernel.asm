[bits 32]

global _start

extern kmain

_start:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    mov esp, 0x00200000
    mov ebp, esp

    call kmain

    jmp $

times (512 - ($ - $$) % 512) db 0
