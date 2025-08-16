#include "hal.h"

uint8_t
PortInByte(uint16_t port)
{
    uint8_t v;
    __asm("inb %1, %0" : "=a"(v) : "d"(port));
    return v;
}

uint16_t
PortInWord(uint16_t port)
{
    uint16_t v;
    __asm("inw %1, %0" : "=a"(v) : "d"(port));
    return v;
}

uint32_t
PortInDword(uint16_t port)
{
    uint32_t v;
    __asm("inl %1, %0" : "=a"(v) : "d"(port));
    return v;
}

void
PortOutByte(uint16_t port, uint8_t v)
{
    __asm("outb %0, %1" : : "a"(v), "d"(port));
}

void
PortOutWord(uint16_t port, uint16_t v)
{
    __asm("outw %0, %1" : : "a"(v), "d"(port));
}

void
PortOutDword(uint16_t port, uint32_t v)
{
    __asm("outl %0, %1" : : "a"(v), "d"(port));
}
