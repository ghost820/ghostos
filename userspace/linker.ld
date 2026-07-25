ENTRY(_start)

PHDRS
{
    executable PT_LOAD FLAGS(5);
    writable PT_LOAD FLAGS(6);
}

SECTIONS
{
    . = 0x00400000;

    .text : ALIGN(4096)
    {
        *(.text .text.*)
        *(.rodata .rodata.*)
    } :executable

    . = ALIGN(4096);

    .data :
    {
        *(.data .data.*)
        *(.bss .bss.*)
        *(COMMON)
    } :writable

    /DISCARD/ :
    {
        *(.eh_frame .eh_frame.*)
        *(.comment)
    }
}
