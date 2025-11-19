#include "hal.h"

uint8_t
PortInByte(uint16_t port)
{
    uint8_t v;
    asm volatile("inb %1, %0" : "=a"(v) : "Nd"(port));
    return v;
}

uint16_t
PortInWord(uint16_t port)
{
    uint16_t v;
    asm volatile("inw %1, %0" : "=a"(v) : "Nd"(port));
    return v;
}

uint32_t
PortInDword(uint16_t port)
{
    uint32_t v;
    asm volatile("inl %1, %0" : "=a"(v) : "Nd"(port));
    return v;
}

void
PortOutByte(uint16_t port, uint8_t v)
{
    asm volatile("outb %0, %1" : : "a"(v), "Nd"(port));
}

void
PortOutWord(uint16_t port, uint16_t v)
{
    asm volatile("outw %0, %1" : : "a"(v), "Nd"(port));
}

void
PortOutDword(uint16_t port, uint32_t v)
{
    asm volatile("outl %0, %1" : : "a"(v), "Nd"(port));
}
