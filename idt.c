#include "idt.h"

InterruptDescriptor32 IDT[512];
InterruptDescriptor32Ptr IDTPtr;

void IDTInit(void) {
    IDTPtr.limit = sizeof(IDT) - 1;
    IDTPtr.base = (uint32_t)IDT;
    IDTSetIDTR(&IDTPtr);
}

void IDTSet(int no, void* addr) {
    IDT[no].offset_1 = (uint32_t)addr & 0xffff;
    IDT[no].selector = 0x08;
    IDT[no].zero = 0x00;
    /*
        bit 1 - enable
        bit 2/3 - ring
    */
    IDT[no].type_attributes = 0x8e;
    IDT[no].offset_2 = (uint32_t)addr >> 16;
}
