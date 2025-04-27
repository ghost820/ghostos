#pragma once

#include "types.h"

#define PAGING_USER_SUPERVISOR 0b00000100
#define PAGING_READWRITE 0b00000010
#define PAGING_PRESENT 0b00000001

void EnablePaging(void);
void SetPageMapping(void* va, void* pa);

uint32_t GetPageDirectoryIndex(void* va);
uint32_t GetPageTableIndex(void* va);
