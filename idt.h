#pragma once

#include "types.h"

#define IDT_SIZE 512

typedef struct __attribute__((packed)) {
   uint16_t offset_1;
   uint16_t selector;
   uint8_t zero;
   uint8_t type_attributes;
   uint16_t offset_2;
} InterruptDescriptor32;

typedef struct __attribute__((packed)) {
   uint16_t limit;
   uint32_t base;
} InterruptDescriptor32Ptr;

typedef struct __attribute__((packed)) {
   uint32_t edi;
   uint32_t esi;
   uint32_t ebp;
   uint32_t oldEsp;
   uint32_t ebx;
   uint32_t edx;
   uint32_t ecx;
   uint32_t eax;
   uint32_t eip;
   uint32_t cs;
   uint32_t eflags;
   uint32_t esp;
   uint32_t ss;
} InterruptFrame;

void IDTInit(void);
extern void EnableInterrupts(void);
extern void DisableInterrupts(void);
extern void IDTSetIDTR(InterruptDescriptor32Ptr* addr);
void IDTSet(int no, void* addr, uint8_t attr);
