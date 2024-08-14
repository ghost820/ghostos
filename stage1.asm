[bits 16]
[org 0x7C00]

; bochs -f <config>
; jmp $ + Ctrl + C
; b <address>
; c
; x <address>
; x/10b <address>
; u <address>
; regs - set = uppercase
; sreg, creg
; page 0

; address = segment * 16 + offset

; Every instruction uses some default segment register
; cs, ds, es, ss in 64-bit mode are always 0

; CS = 0x07C0, IP = 0x0000
; OR
; CS = 0x0000, IP = 0x7C00
jmp word 0x0000:start
start:

; Load at 0x20000 (0x2000 * 16)
mov ax, 0x2000
mov es, ax
mov bx, 0

mov ah, 2
mov al, 64 ; number of sectors to read
mov ch, 0
mov cl, 2  ; from sector 2
mov dh, 0
; DL already contains the boot drive number
int 13h
; CF on error, AH = status, AL = number of sectors read

jmp word 0x2000:0x0000

%if ($ - $$) > 510
    %fatal "Bootloader code exceeds 512 bytes."
%endif

times 510 - ($ - $$) db 0
db 0x55
db 0xAA
