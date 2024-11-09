[bits 16]
[org 0x7c00]

BPB:
    jmp short start
    nop
    times 33 db 0

start:
    ; Set code segment
    jmp 0x0000:_16

_16:
    cli

    mov ax, 0x0000
    mov ds, ax
    mov es, ax
    mov ss, ax
    
    mov sp, 0x7c00

    ; Enable A20
    in al, 0x92
    or al, 2
    out 0x92, al

    lgdt [GDT_ADDR]

    mov eax, cr0
    or eax, 1
    mov cr0, eax

    jmp 0x08:_32

[bits 32]
_32:
    mov eax, 1
    mov ecx, 100
    mov edi, 0x100000
    call ata_read

    jmp 0x08:0x100000
    
; eax - first sector
; ecx - number of sectors
; edi - destination address
ata_read:
    mov ebx, eax

    shr eax, 24
    or eax, 0xe0 ; Master drive
    mov dx, 0x01f6
    out dx, al

    mov eax, ecx
    mov dx, 0x01f2
    out dx, al

    mov eax, ebx
    mov dx, 0x01f3
    out dx, al

    shr eax, 8
    mov dx, 0x01f4
    out dx, al

    mov eax, ebx
    shr eax, 16
    mov dx, 0x01f5
    out dx, al

    mov al, 0x20
    mov dx, 0x01f7
    out dx, al

    .loop:
        .wait_until_ready:
            mov dx, 0x01f7
            in al, dx
            test al, 8
        jz .wait_until_ready

        push ecx
        mov ecx, 256
        mov dx, 0x01f0
        rep insw
        pop ecx
    loop .loop

    ret

GDT:
    ; Null descriptor
    dd 0
    dd 0

    ; Code descriptor
    dw 0xffff
    dw 0x0000
    db 0x00
    db 0x9a
    db 11001111b
    db 0x00

    ; Data descriptor
    dw 0xffff
    dw 0x0000
    db 0x00
    db 0x92
    db 11001111b
    db 0x00
GDT_END:

GDT_ADDR:
    dw GDT_END - GDT - 1
    dd GDT

%if ($ - $$) > 510
    %fatal "Bootloader code exceeds 512 bytes."
%endif

times 510 - ($ - $$) db 0
dw 0xAA55
