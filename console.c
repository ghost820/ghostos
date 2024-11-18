#include "console.h"

#include <stdarg.h>

#include "memory.h"
#include "hal.h"

#define CONSOLE_WIDTH 80
#define CONSOLE_HEIGHT 25

typedef struct {
    int x;
    int y;
} ConsoleCtx_t;
static ConsoleCtx_t ConsoleCtx;

void ConsoleInit(void) {
    ConsoleCtx.x = 0;
    ConsoleCtx.y = 0;
}

void PutChar(char c) {
    PutCharC(CONSOLE_WHITE, c);
}

void PutCharC(ConsoleColor color, char c) {
    char* VRAM = (char*)0xB8000;

    if (ConsoleCtx.y == CONSOLE_HEIGHT) {
        ScrollLine();
    }

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
    VRAM[offset * 2 + 1] = color;

    x += 1;
    if (x == CONSOLE_WIDTH) {
        x = 0;
        y += 1;
    }
    SetCursorPosition(x, y);
}

void Print(const char* str, ...) {
    va_list args;
    va_start(args, str);

    for (; *str != '\0'; str++) {
        if (*str != '%') {
            PutChar(*str);
            continue;
        }

        switch(*++str) {
            case '%':
                PutChar('%');
                break;

            case 'd': {
                int32_t n = va_arg(args, int32_t);

                if (n < 0) {
                    PutChar('-');
                    n = -n;
                }

                char buf[16];
                int i = 0;
                do {
                    buf[i++] = '0' + n % 10;
                    n /= 10;
                } while (n > 0);

                for (i--; i >= 0; i--) {
                    PutChar(buf[i]);
                }
                break;
            }

            case 'c':
                PutChar(va_arg(args, int));
                break;

            case 's':
                for (const char* s = va_arg(args, const char*); *s != '\0'; s++) {
                    PutChar(*s);
                }
                break;
        }
    }

    va_end(args);
}

void PrintC(ConsoleColor color, const char* str, ...) {
    va_list args;
    va_start(args, str);

    for (; *str != '\0'; str++) {
        if (*str != '%') {
            PutCharC(color, *str);
            continue;
        }

        switch(*++str) {
            case '%':
                PutCharC(color, '%');
                break;

            case 'd': {
                int32_t n = va_arg(args, int32_t);

                if (n < 0) {
                    PutCharC(color, '-');
                    n = -n;
                }

                char buf[16];
                int i = 0;
                do {
                    buf[i++] = '0' + n % 10;
                    n /= 10;
                } while (n > 0);

                for (i--; i >= 0; i--) {
                    PutCharC(color, buf[i]);
                }
                break;
            }

            case 'c':
                PutCharC(color, va_arg(args, int));
                break;

            case 's':
                for (const char* s = va_arg(args, const char*); *s != '\0'; s++) {
                    PutCharC(color, *s);
                }
                break;
        }
    }

    va_end(args);
}

void ScrollLine(void) {
    char* VRAM = (char*)0xB8000;

    memmove(VRAM, VRAM + CONSOLE_WIDTH * 2, CONSOLE_WIDTH * (CONSOLE_HEIGHT - 1) * 2);
    int i = CONSOLE_WIDTH * (CONSOLE_HEIGHT - 1) * 2;
    while (i < CONSOLE_WIDTH * CONSOLE_HEIGHT * 2) {
        VRAM[i + 0] = ' ';
        VRAM[i + 1] = 0x07;
        i += 2;
    }

    SetCursorPosition(0, CONSOLE_HEIGHT - 1);
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
