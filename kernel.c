#include "kernel.h"

#include "gdt.h"
#include "idt.h"
#include "heap.h"
#include "paging.h"
#include "console.h"

void kmain()
{
    GDTInit();
    IDTInit();
    HeapInit();
    EnablePaging();
    EnableInterrupts();
    ConsoleInit();

    ClearScreen();
    Print("GhostOS");
}
