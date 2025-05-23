section .asm

global EnablePagingAsm
global SetPageDirectoryAsm

EnablePagingAsm:
    push ebp
    mov ebp, esp

    mov eax, [ebp + 8]
    mov cr3, eax

    mov eax, cr0
    or eax, 0x80000000
    mov cr0, eax

    pop ebp
    ret

SetPageDirectoryAsm:
    push ebp
    mov ebp, esp

    mov eax, [ebp + 8]
    mov cr3, eax

    pop ebp
    ret
