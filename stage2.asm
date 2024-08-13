[bits 16]
[org 0x0000]

mov ax, 0x2000
mov ds, ax
mov es, ax

mov ax, 0x1F00
mov ss, ax
mov sp, 0

; mov ax, 0xB800 ; 0xB8000
; mov fs, ax
; mov bx, 0
; mov ax, 0x4141 ; first byte is color
; mov [fs:bx], ax

lgdt [GDT_addr]

mov eax, cr0
or eax, 1
mov cr0, eax

jmp dword 0x8:(0x20000+start32) ; 0x8 = code segment

start32:
[bits 32]

mov ax, 0x10
mov ds, ax
mov es, ax
mov ss, ax

lea eax, [0xb8000]
mov dword [eax], 0x41414141

jmp $

GDT_addr:
    dw (GDT_end - GDT) - 1 ; Size bitmask if number of segments is power of 2
    dd 0x20000 + GDT

times (32 - ($ - $$) % 32) db 0xCC
GDT:
    ; Null segment
    dd 0, 0

    ; Code segment
    dd 0xFFFF ; base address (0) + segment limit
    dd (10 << 8) | (1 << 12) | (1 << 15) | (0xF << 16) | (1 << 22) | (1 << 23)

    ; Data segment
    dd 0xFFFF ; base address (0) + segment limit
    dd (2 << 8) | (1 << 12) | (1 << 15) | (0xF << 16) | (1 << 22) | (1 << 23)

    ; Null segment
    dd 0, 0
GDT_end:

%if ($ - $$) > 4096
    %fatal "Bootloader code exceeds 4096 bytes."
%endif
