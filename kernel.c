#include "kernel.h"

#include "idt.h"
#include "heap.h"
#include "paging.h"
#include "console.h"

void kmain()
{
    IDTInit();
    HeapInit();
    EnablePaging();
    EnableInterrupts();
    ConsoleInit();

    ClearScreen();
    Print("GhostOS");
}
