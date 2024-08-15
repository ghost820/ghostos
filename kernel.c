_Noreturn void _start(void) {
    short* VRAM = (short*)0xB8000;

    VRAM[0] = 0x4141;

    while (1);
}
