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

    ; Remap the master PIC to interrupt 0x20
    mov al, 0x11 ; Init command
    out 0x20, al

    mov al, 0x20
    out 0x21, al

    mov al, 0x01 ; Mode ICW4
    out 0x21, al

    sti

    call kmain

    jmp $

times (512 - ($ - $$) % 512) db 0
