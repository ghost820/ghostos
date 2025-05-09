#pragma once

#include "types.h"

typedef struct __attribute__((packed)) {
    uint16_t link;
    uint16_t _res0;
    uint32_t esp0;
    uint16_t ss0;
    uint16_t _res1;
    uint32_t esp1;
    uint16_t ss1;
    uint16_t _res2;
    uint32_t esp2;
    uint16_t ss2;
    uint16_t _res3;
    uint32_t cr3;
    uint32_t eip;
    uint32_t eflags;
    uint32_t eax;
    uint32_t ecx;
    uint32_t edx;
    uint32_t ebx;
    uint32_t esp;
    uint32_t ebp;
    uint32_t esi;
    uint32_t edi;
    uint16_t es;
    uint16_t _res4;
    uint16_t cs;
    uint16_t _res5;
    uint16_t ss;
    uint16_t _res6;
    uint16_t ds;
    uint16_t _res7;
    uint16_t fs;
    uint16_t _res8;
    uint16_t gs;
    uint16_t _res9;
    uint16_t ldtr;
    uint16_t _res10;
    uint16_t _res11;
    uint16_t iopb;
    uint32_t ssp;
} TSS;

extern TSS tss;

void TSSInit(void);
extern void TSSLoad(uint16_t segmentSelector);
