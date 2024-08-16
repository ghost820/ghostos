#include "hal.h"

void PortOutByte(uint16_t port, uint8_t v) {
    __asm("outb %0, %1" : : "a" (v), "d" (port));
}

void PortOutWord(uint16_t port, uint16_t v) {
    __asm("outw %0, %1" : : "a" (v), "d" (port));
}

void PortOutDword(uint16_t port, uint32_t v) {
    __asm("outl %0, %1" : : "a" (v), "d" (port));
}
