#include "console.h"

#include "hal.h"

#define CONSOLE_WIDTH 80
#define CONSOLE_HEIGHT 25

typedef struct {
    int x;
    int y;
} ConsoleCtx_t;
static ConsoleCtx_t ConsoleCtx;

void PutChar(char c) {
    char* VRAM = (char*)0xB8000;

    int x = ConsoleCtx.x;
    int y = ConsoleCtx.y;

    if (c == '\n') {
        SetCursorPosition(0, ConsoleCtx.y + 1);
        return;
    }

    if (c == '\t') {
        x += 8 - x % 8;
        if (x >= CONSOLE_WIDTH) {
            x = 0;
            y += 1;
        }
        SetCursorPosition(x, y);
        return;
    }

    int offset = x + y * CONSOLE_WIDTH;
    VRAM[offset * 2 + 0] = c;
    VRAM[offset * 2 + 1] = 0x07;

    x += 1;
    if (x == CONSOLE_WIDTH) {
        x = 0;
        y += 1;
    }
    SetCursorPosition(x, y);
}

void Print(const char* str) {
    for (; *str != '\0'; str++) {
        PutChar(*str);
    }
}

void SetCursorPosition(int x, int y) {
    ConsoleCtx.x = x;
    ConsoleCtx.y = y;

    uint16_t offset = x + y * CONSOLE_WIDTH;
    PortOutByte(0x3D4, 0x0F);
    PortOutByte(0x3D5, (uint8_t)offset);
    PortOutByte(0x3D4, 0x0E);
    PortOutByte(0x3D5, (uint8_t)(offset >> 8));
}

void ClearScreen(void) {
    char* VRAM = (char*)0xB8000;

    for (int i = 0; i < CONSOLE_WIDTH * CONSOLE_HEIGHT; i++) {
        VRAM[i * 2 + 0] = ' ';
        VRAM[i * 2 + 1] = 0x07;
    }

    SetCursorPosition(0, 0);
}
