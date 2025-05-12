#pragma once

#include "types.h"

void* memset(void* dst, uint8_t c, uint32_t size);
void* memcpy(void* dst, const void* src, uint32_t size);
void* memmove(void* dst, const void* src, uint32_t size);
