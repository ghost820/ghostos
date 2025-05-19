#include "string.h"

char* strncpy(char* dst, const char* src, uint32_t count) {
    uint32_t i;
    for (i = 0; i < count && src[i] != '\0'; i++) {
        dst[i] = src[i];
    }
    for (; i < count; i++) {
        dst[i] = '\0';
    }
    return dst;
}
