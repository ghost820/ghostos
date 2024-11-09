#include "memory.h"

void* memset(void* dst, uint8_t c, uint64_t size) {
    for (uint64_t i = 0; i < size; i++) {
        ((uint8_t*)dst)[i] = c;
    }
    return dst;
}

void* memmove(void* dst, const void* src, uint64_t size) {
    if (dst < src) {
        for (uint64_t i = 0; i < size; i++) {
            ((uint8_t*)dst)[i] = ((const uint8_t*)src)[i];
        }
    } else {
        uint64_t i = size - 1;
        while (1) {
            ((uint8_t*)dst)[i] = ((const uint8_t*)src)[i];
            if (i == 0) {
                break;
            }
            i--;
        }
    }
    return dst;
}
