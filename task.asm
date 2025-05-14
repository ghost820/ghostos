section .asm

global GoToUserMode
global SetSegmentRegistersToUser
global SetRegisters

GoToUserMode:
    mov ebp, esp

    mov ebx, [ebp + 4]

    push dword [ebx + 36]
    push dword [ebx + 28]

    mov eax, [ebx + 40]
    or eax, 0x200
    push eax

    push dword [ebx + 32]
    push dword [ebx + 44]

    mov ax, [ebx + 36]
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    push dword [ebp + 4]
    call SetRegisters
    add esp, 4

    iretd

SetSegmentRegistersToUser:
    mov ax, 0x23
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ret

SetRegisters:
    push ebp
    mov ebp, esp

    mov ebx, [ebp + 8]
    mov eax, [ebx]
    mov ecx, [ebx + 8]
    mov edx, [ebx + 12]
    mov esi, [ebx + 16]
    mov edi, [ebx + 20]
    mov ebp, [ebx + 24]
    mov ebx, [ebx + 4]

    add esp, 4
    ret
