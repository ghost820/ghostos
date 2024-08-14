void _start() {
    short* VRAM = 0xB8000;

    VRAM[0] = 0x4141;

    while (1);
}
