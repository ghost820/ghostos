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

jmp dword 0x8:(0x20000+start32) ; 0x8 = code segment, 0x8 >> 3 = 1, last three bits are ring level

start32:
[bits 32]

mov ax, 0x10
mov ds, ax
mov es, ax
mov ss, ax

mov eax, (PML4 - $$) + 0x20000
mov cr3, eax

mov eax, cr4
or eax, 1 << 5
mov cr4, eax

mov ecx, 0xC0000080
rdmsr
or eax, 1 << 8
wrmsr

mov eax, cr0
or eax, 1 << 31
mov cr0, eax

lgdt [GDT64_addr + 0x20000]

jmp dword 0x8:(0x20000+start64)

start64:
[bits 64]

mov ax, 0x10
mov ds, ax
mov es, ax
mov ss, ax

; lea rax, [0xb8000]
; mov dword [rax], 0x41414141

; jmp $

loader:
mov rsi, [0x20000 + kernel + 0x20] ; e_phoff
add rsi, 0x20000 + kernel
movzx ecx, word [0x20000 + kernel + 0x38] ; e_phnum
cld
.ph_loop: ; local to loader
    mov eax, [rsi + 0x00]
    cmp eax, 1 ; p_type != PT_LOAD
    jne .next

    mov r8, [rsi + 0x08] ; p_offset
    mov r9, [rsi + 0x10] ; p_vaddr
    mov r10, [rsi + 0x20] ; p_filesz

    ; Backup
    mov rbp, rsi
    mov r15, rcx

    ; Copy segment
    lea rsi, [0x20000 + kernel + r8d]
    mov rdi, r9
    mov rcx, r10
    rep movsb

    ; Restore
    mov rcx, r15
    mov rsi, rbp
.next:
add rsi, 0x20 ; sizeof(Elf64_Phdr)
loop .ph_loop

; Fix stack
mov rsp, 0x30F000

; Jump to EP
mov rax, [0x20000 + kernel + 0x18]
add rax, 0x20000 + kernel
call rax

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

GDT64_addr:
    dw (GDT64_end - GDT64) - 1
    dd 0x20000 + GDT64

times (32 - ($ - $$) % 32) db 0xCC
GDT64:
    dd 0, 0

    dd 0xFFFF
    dd (10 << 8) | (1 << 12) | (1 << 15) | (0xF << 16) | (1 << 21) | (1 << 23)

    dd 0xFFFF
    dd (2 << 8) | (1 << 12) | (1 << 15) | (0xF << 16) | (1 << 21) | (1 << 23)

    dd 0, 0
GDT64_end:

; Page tables
times (4096 - ($ - $$) % 4096) db 0
PML4:
    dq 1 | (1 << 1) | (PDPTE - $$ + 0x20000)
    times 511 dq 0

; Assume: aligned to 4KB
PDPTE:
    dq 1 | (1 << 1) | (1 << 7)
    times 511 dq 0

times (512 - ($ - $$) % 512) db 0

%if ($ - $$) > 16384
    %fatal "Bootloader code exceeds 16384 bytes."
%endif

kernel:
