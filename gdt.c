#include "gdt.h"

uint64_t GDT[3];

void GDTInit(void) {
    GDT[0] = GDTCreateEntry(0, 0, 0, GDT_BYTES);                   // Null descriptor
    GDT[1] = GDTCreateEntry(0, 0xffffffff, 0b10011010, GDT_PAGES); // Code descriptor
    GDT[2] = GDTCreateEntry(0, 0xffffffff, 0b10010010, GDT_PAGES); // Data descriptor
    GDTLoad(GDT, sizeof(GDT));
}

uint64_t GDTCreateEntry(const void* base, uint32_t limit, uint8_t access, GDTUnit unit) {
    uint64_t result = 0;
    uint8_t* resultBytes = (uint8_t*)&result;

    // Encode flags
    resultBytes[6] = 0b01000000;
    if (unit == GDT_PAGES) {
        resultBytes[6] = 0b11000000;
    }

    // Encode limit
    resultBytes[0] = limit & 0xff;
    resultBytes[1] = (limit >> 8) & 0xff;
    resultBytes[6] |= (limit >> 16) & 0x0f;

    // Encode base
    resultBytes[2] = (uint32_t)base & 0xff;
    resultBytes[3] = ((uint32_t)base >> 8) & 0xff;
    resultBytes[4] = ((uint32_t)base >> 16) & 0xff;
    resultBytes[7] = ((uint32_t)base >> 24) & 0xff;

    // Encode access
    resultBytes[5] = access;

    return result;
}
