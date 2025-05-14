#pragma once

#include "types.h"

#define PAGING_USER_SUPERVISOR 0b00000100
#define PAGING_READWRITE 0b00000010
#define PAGING_PRESENT 0b00000001

void EnablePaging(void);
uint32_t* CreatePageDirectory(uint16_t dirFlags, uint16_t pageFlags);
void SetPageDirectory(uint32_t* pageDirectory);
void SetPageMapping(void* va, void* pa, uint16_t flags);
void FreePageDirectory(uint32_t* pageDirectory);

uint32_t GetPageDirectoryIndex(void* va);
uint32_t GetPageTableIndex(void* va);

uint32_t BytesToPages(uint32_t bytes);
