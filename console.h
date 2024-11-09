#pragma once

#include "types.h"

typedef enum {
    CONSOLE_BLACK,
    CONSOLE_BLUE,
    CONSOLE_GREEN,
    CONSOLE_CYAN,
    CONSOLE_RED,
    CONSOLE_PURPLE,
    CONSOLE_BROWN,
    CONSOLE_GRAY,
    CONSOLE_DARKGRAY,
    CONSOLE_LIGHTBLUE,
    CONSOLE_LIGHTGREEN,
    CONSOLE_LIGHTCYAN,
    CONSOLE_LIGHTRED,
    CONSOLE_LIGHTPURPLE,
    CONSOLE_YELLOW,
    CONSOLE_WHITE
} ConsoleColor;

void PutChar(char c);
void PutCharC(ConsoleColor color, char c);
void Print(const char* str, ...);
void PrintC(ConsoleColor color, const char* str, ...);
void ScrollLine(void);
void SetCursorPosition(int x, int y);
void ClearScreen(void);
