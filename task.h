#pragma once

#include "types.h"

typedef struct {
    uint32_t eax;
    uint32_t ebx;
    uint32_t ecx;
    uint32_t edx;
    uint32_t esi;
    uint32_t edi;
    uint32_t ebp;
    uint32_t esp;

    uint32_t cs;
    uint32_t ss;

    uint32_t eflags;

    uint32_t eip;
} TaskRegisters;

typedef struct Task {
    TaskRegisters registers;
    uint32_t* pageDirectory;

    struct Task* prev;
    struct Task* next;
} Task;

Task* TaskInit(void);
void TaskFree(Task* task);
