[bits 32]

global _start

extern kmain

_start:
    mov esi, eax
    mov edi, ebx

    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    mov esp, 0x00200000
    mov ebp, esp

    ; Master and slave PICs
    mov al, 0x11 ; Start the initialization sequence in cascade mode
    out 0x20, al
    mov al, 0    ; Wait a moment
    out 0x80, al

    mov al, 0x11
    out 0xA0, al
    mov al, 0
    out 0x80, al

    mov al, 0x20 ; Set offsets
    out 0x21, al
    mov al, 0
    out 0x80, al

    mov al, 0x28
    out 0xA1, al
    mov al, 0
    out 0x80, al

    mov al, 0x04 ; Tell master PIC that there is a slave PIC at IRQ2
    out 0x21, al
    mov al, 0
    out 0x80, al

    mov al, 0x02 ; Tell slave PIC its cascade identity
    out 0xA1, al
    mov al, 0
    out 0x80, al

    mov al, 0x01 ; Set 8086 mode
    out 0x21, al
    mov al, 0
    out 0x80, al

    mov al, 0x01
    out 0xA1, al
    mov al, 0
    out 0x80, al

    mov al, 0x00 ; Unmask
    out 0x21, al

    mov al, 0x00
    out 0xA1, al

    push esi
    push edi
    call kmain

    ; jmp $
    cli
    hlt

; Multiboot header
align 4
dd 0x1BADB002
dd 0x3
dd -(0x1BADB002 + 0x3)

times (512 - ($ - $$) % 512) db 0
