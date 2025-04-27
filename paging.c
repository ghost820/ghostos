#include "paging.h"

#include "heap.h"

uint32_t* PAGE_DIRECTORY;

extern void EnablePagingAsm(uint32_t* directory);

void EnablePaging(void) {
    uint32_t* directory = kzalloc(1024 * 4);
    uint64_t offset = 0;
    for (int i = 0; i < 1024; i++)
    {
        uint32_t* entry = kzalloc(1024 * 4);
        for (int j = 0; j < 1024; j++)
        {
            entry[j] = (offset + j * 4096) & 0xfffff000;
            entry[j] |= PAGING_PRESENT | PAGING_READWRITE;
        }
        directory[i] = (uint32_t)entry & 0xfffff000;
        directory[i] |= PAGING_PRESENT | PAGING_READWRITE;
        offset += 1024 * 4096;
    }
    
    EnablePagingAsm(directory);

    PAGE_DIRECTORY = directory;
}

void SetPageMapping(void* va, void* pa) {
    uint32_t dirIdx = GetPageDirectoryIndex(va);
    uint32_t tblIdx = GetPageTableIndex(va);
    uint32_t* pageTable = (uint32_t*)(PAGE_DIRECTORY[dirIdx] & 0xfffff000);
    pageTable[tblIdx] = (uint32_t)pa | PAGING_PRESENT | PAGING_READWRITE;
}

uint32_t GetPageDirectoryIndex(void* va) {
    return (uint32_t)va / (1024 * 4096);
}

uint32_t GetPageTableIndex(void* va) {
    return (uint32_t)va % (1024 * 4096) / 4096;
}
