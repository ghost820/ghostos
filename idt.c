#include "idt.h"

#include "hal.h"
#include "paging.h"
#include "task.h"

/*
    PortOutByte(0x20, 0x20):
      * Needed only for PIC interrupts.
      * Send to both PICs in case of slave.
*/

extern void int20h(void);
extern void int21h(void);
extern void int80h(void);
extern void intdh(void);

InterruptDescriptor32 IDT[IDT_SIZE];
InterruptDescriptor32Ptr IDTPtr;

void
IDTInit(void)
{
    for (int i = 0; i < IDT_SIZE; i++) {
        IDTSet(i, intdh, 0x8e);
    }
    IDTSet(0x20, int20h, 0x8e); // Timer
    IDTSet(0x21, int21h, 0x8e); // Keyboard
    IDTSet(0x80, int80h, 0xee); // Syscall

    IDTPtr.limit = sizeof(IDT) - 1;
    IDTPtr.base = (uint32_t)IDT;
    IDTSetIDTR(&IDTPtr);
}

void
IDTSet(int no, void* addr, uint8_t attr)
{
    IDT[no].offset_1 = (uint32_t)addr & 0xffff;
    IDT[no].selector = 0x08;
    IDT[no].zero = 0x00;
    /*
        bit 1 - enable
        bit 2/3 - ring
    */
    IDT[no].type_attributes = attr;
    IDT[no].offset_2 = (uint32_t)addr >> 16;
}

void
int20h_handler(void)
{
    // Code...
    PortOutByte(0x20, 0x20);
}

void
int21h_handler(void)
{
    // Code...
    PortOutByte(0x20, 0x20);
}

void
int80h_handler(int command, InterruptFrame* frame)
{
    SetSegmentRegistersToKernel();
    SetPageDirectory(PAGE_DIRECTORY_KERNEL);

    CurrentTask->registers.edi = frame->edi;
    CurrentTask->registers.esi = frame->esi;
    CurrentTask->registers.ebp = frame->ebp;
    CurrentTask->registers.ebx = frame->ebx;
    CurrentTask->registers.edx = frame->edx;
    CurrentTask->registers.ecx = frame->ecx;
    CurrentTask->registers.eax = frame->eax;
    CurrentTask->registers.eip = frame->eip;
    CurrentTask->registers.cs = frame->cs;
    CurrentTask->registers.eflags = frame->eflags;
    CurrentTask->registers.esp = frame->esp;
    CurrentTask->registers.ss = frame->ss;

    SetSegmentRegistersToUser();
    SetPageDirectory(CurrentTask->pageDirectory);
}

void
intdh_handler(void)
{
    // Code...
    PortOutByte(0x20, 0x20);
}
