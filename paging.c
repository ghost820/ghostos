#include "paging.h"

#include "heap.h"

uint32_t* PAGE_DIRECTORY;

extern void EnablePagingAsm(uint32_t* directory);
extern void SetPageDirectoryAsm(uint32_t* directory);

void EnablePaging(void) {
    PAGE_DIRECTORY = CreatePageDirectory(
        PAGING_PRESENT | PAGING_READWRITE,
        PAGING_PRESENT | PAGING_READWRITE
    );
    EnablePagingAsm(PAGE_DIRECTORY);
}

uint32_t* CreatePageDirectory(uint16_t dirFlags, uint16_t pageFlags) {
    uint32_t* directory = kzalloc(1024 * 4);
    uint64_t offset = 0;
    for (int i = 0; i < 1024; i++)
    {
        uint32_t* entry = kzalloc(1024 * 4);
        for (int j = 0; j < 1024; j++)
        {
            entry[j] = (offset + j * 4096) & 0xfffff000;
            entry[j] |= pageFlags;
        }
        directory[i] = (uint32_t)entry & 0xfffff000;
        directory[i] |= dirFlags;
        offset += 1024 * 4096;
    }
    return directory;
}

void SetPageDirectory(uint32_t* pageDirectory) {
    PAGE_DIRECTORY = pageDirectory;
    SetPageDirectoryAsm(pageDirectory);
}

void SetPageMapping(uint32_t* pageDirectory, void* va, void* pa, uint16_t flags) {
    uint32_t dirIdx = GetPageDirectoryIndex(va);
    uint32_t tblIdx = GetPageTableIndex(va);
    uint32_t* pageTable = (uint32_t*)(pageDirectory[dirIdx] & 0xfffff000);
    pageTable[tblIdx] = (uint32_t)pa | flags;
}

void FreePageDirectory(uint32_t* pageDirectory) {
    for (int i = 0; i < 1024; i++) {
        uint32_t* entry = (uint32_t*)(pageDirectory[i] & 0xfffff000);
        kfree(entry);
    }
    kfree(pageDirectory);
}

uint32_t GetPageDirectoryIndex(void* va) {
    return (uint32_t)va / (1024 * 4096);
}

uint32_t GetPageTableIndex(void* va) {
    return (uint32_t)va % (1024 * 4096) / 4096;
}

uint32_t BytesToPages(uint32_t bytes) {
    return (bytes + 4095) / 4096;
}
