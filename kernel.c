#include "kernel.h"

#include "idt.h"
#include "heap.h"
#include "console.h"

void kmain()
{
    IDTInit();
    EnableInterrupts();
    HeapInit();
    ConsoleInit();

    ClearScreen();
    Print("GhostOS");
}
