#include "src/console.h"

_Noreturn void _start(void) {
    ClearScreen();
    Print("GhostOS");

    while (1);
}
