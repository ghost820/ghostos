#include "kernel.h"

#include "idt.h"
#include "console.h"

void kmain()
{
    IDTInit();

    ClearScreen();
    Print("GhostOS");
}
