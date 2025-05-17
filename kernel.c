#include "kernel.h"

#include "gdt.h"
#include "tss.h"
#include "idt.h"
#include "heap.h"
#include "paging.h"
#include "console.h"
#include "process.h"
#include "task.h"

void kmain()
{
    GDTInit();
    IDTInit();
    TSSInit();
    HeapInit();
    EnablePaging();
    ConsoleInit();
    ProcessEnvironmentInit();

    ClearScreen();
    Print("GhostOS");

    uint8_t processImage[] = {
        0xeb, 0xfe
    };
    Process* process = ProcessInit(processImage, sizeof(processImage));
    SetCurrentTask(process->mainThread);
}
