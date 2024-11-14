#include "idt.h"

#include "hal.h"

extern void int20h(void);
extern void int21h(void);
extern void intdh(void);

InterruptDescriptor32 IDT[IDT_SIZE];
InterruptDescriptor32Ptr IDTPtr;

void IDTInit(void) {
    for (int i = 0; i < IDT_SIZE; i++) {
        IDTSet(i, intdh);
    }
    IDTSet(0x20, int20h); // Timer
    IDTSet(0x21, int21h); // Keyboard

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

void int20h_handler(void) {
    // Code...
    PortOutByte(0x20, 0x20);
}

void int21h_handler(void) {
    // Code...
    PortOutByte(0x20, 0x20);
}

void intdh_handler(void) {
    // Code...
    PortOutByte(0x20, 0x20);
}
