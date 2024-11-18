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

void IDTInit(void);
extern void EnableInterrupts(void);
extern void DisableInterrupts(void);
extern void IDTSetIDTR(InterruptDescriptor32Ptr* addr);
void IDTSet(int no, void* addr);
