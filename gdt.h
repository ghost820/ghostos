#pragma once

#include "types.h"

typedef enum {
    GDT_BYTES,
    GDT_PAGES
} GDTUnit;

void GDTInit(void);
extern void GDTLoad(void* gdt, uint16_t size);
uint64_t GDTCreateEntry(const void* base, uint32_t limit, uint8_t access, GDTUnit unit);
