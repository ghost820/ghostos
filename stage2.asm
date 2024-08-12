[bits 16]
[org 0x0000]

mov ax, 0x2000
mov ds, ax
mov es, ax

mov ax, 0x1F00
mov ss, ax
mov sp, 0

mov ax, 0xB800 ; 0xB8000
mov fs, ax
mov bx, 0
mov ax, 0x4141 ; first byte is color
mov [fs:bx], ax

jmp $

%if ($ - $$) > 4096
    %fatal "Bootloader code exceeds 4096 bytes."
%endif
