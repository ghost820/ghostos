#include "heap.h"

#include "memory.h"

uint8_t HEAP_TABLE[HEAP_SIZE / HEAP_BLOCK_SIZE];

void
HeapInit(void)
{
    for (uint32_t i = 0; i < HEAP_SIZE / HEAP_BLOCK_SIZE; i++) {
        HEAP_TABLE[i] = 0;
    }
}

void*
kmalloc(uint32_t size)
{
    uint32_t allocSize = ((size + HEAP_BLOCK_SIZE - 1) / HEAP_BLOCK_SIZE) * HEAP_BLOCK_SIZE;

    uint32_t startBlockIdx = 0xffffffff;
    uint32_t numOfFreeBlocks = 0;
    for (uint32_t i = 0; i < HEAP_SIZE / HEAP_BLOCK_SIZE; i++) {
        if ((HEAP_TABLE[i] & 0x0f) != 0) {
            startBlockIdx = 0xffffffff;
            numOfFreeBlocks = 0;
            continue;
        }

        if (startBlockIdx == 0xffffffff) {
            startBlockIdx = i;
        }

        numOfFreeBlocks++;
        if (numOfFreeBlocks == allocSize / HEAP_BLOCK_SIZE) {
            break;
        }
    }
    if (startBlockIdx == 0xffffffff || numOfFreeBlocks < allocSize / HEAP_BLOCK_SIZE) {
        return 0;
    }

    HEAP_TABLE[startBlockIdx] = numOfFreeBlocks > 1 ? 0b10000001 : 0b00000001;
    for (uint32_t i = 1; i < numOfFreeBlocks - 1; i++) {
        HEAP_TABLE[startBlockIdx + i] = 0b10000001;
    }
    HEAP_TABLE[startBlockIdx + numOfFreeBlocks - 1] = 0b00000001;

    return (void*)(HEAP_ADDR + startBlockIdx * HEAP_BLOCK_SIZE);
}

void*
kzalloc(uint32_t size)
{
    void* mem = kmalloc(size);
    if (mem) {
        memset(mem, 0, size);
    }
    return mem;
}

void
kfree(void* ptr)
{
    uint32_t blockIdx = ((uint32_t)ptr - HEAP_ADDR) / HEAP_BLOCK_SIZE;
    while (1) {
        uint8_t entry = HEAP_TABLE[blockIdx];
        HEAP_TABLE[blockIdx] = 0;
        if ((entry & 0b10000000) == 0) {
            break;
        }
        blockIdx++;
    }
}
