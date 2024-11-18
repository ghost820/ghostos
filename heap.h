#pragma once

#include "types.h"

#define HEAP_ADDR 0x01000000
#define HEAP_SIZE 104857600
#define HEAP_BLOCK_SIZE 4096

void HeapInit(void);

void* kmalloc(uint32_t size);
void kfree(void* ptr);
