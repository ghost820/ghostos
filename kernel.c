#include "kernel.h"

#include "gdt.h"
#include "tss.h"
#include "idt.h"
#include "heap.h"
#include "paging.h"
#include "console.h"
#include "process.h"

void kmain()
{
    GDTInit();
    IDTInit();
    TSSInit();
    HeapInit();
    EnablePaging();
    EnableInterrupts();
    ConsoleInit();
    ProcessEnvironmentInit();

    ClearScreen();
    Print("GhostOS");
}
