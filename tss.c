#include "tss.h"

#include "memory.h"

TSS tss;

void TSSInit(void) {
    memset(&tss, 0, sizeof(TSS));
    tss.esp0 = 0x600000;
    tss.ss0 = 0x10;
    TSSLoad(0x28);
}
