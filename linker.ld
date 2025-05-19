OUTPUT_FORMAT(binary)

ENTRY(_start)

SECTIONS
{
    /* Sections at address */
    . = 1M;

    .text : ALIGN(4096)
    {
        *(.text)
    }

    /* To avoid alignment issues */
    .asm : ALIGN(4096)
    {
        *(.asm)
    }

    .rodata : ALIGN(4096)
    {
        *(.rodata)
    }

    .data : ALIGN(4096)
    {
        *(.data)
    }

    .bss : ALIGN(4096)
    {
        *(COMMON)
        *(.bss)
    }
}
