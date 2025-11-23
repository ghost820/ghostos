[bits 16]

%include "../constants.inc"

global entry

extern _kmain16

section _ENTRY class=CODE

entry:
    cli

    mov ax, KERNEL16_SEGMENT
    mov ds, ax
    mov es, ax
    mov ss, ax

    xor sp, sp
    xor bp, bp

    call _kmain16

    cli
    hlt
