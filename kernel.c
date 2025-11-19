#include "kernel.h"

#include "console.h"
#include "gdt.h"
#include "heap.h"
#include "idt.h"
#include "keyboard.h"
#include "multiboot.h"
#include "paging.h"
#include "process.h"
#include "task.h"
#include "tss.h"

void
kmain(multiboot_info_t* mbd, uint32_t magic)
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

    KeyboardInit();
}
