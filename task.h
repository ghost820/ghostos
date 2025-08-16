#pragma once

#include "types.h"

typedef struct {
    uint32_t eax; // 0
    uint32_t ebx; // 4
    uint32_t ecx; // 8
    uint32_t edx; // 12
    uint32_t esi; // 16
    uint32_t edi; // 20
    uint32_t ebp; // 24
    uint32_t esp; // 28

    uint32_t cs; // 32
    uint32_t ss; // 36

    uint32_t eflags; // 40

    uint32_t eip; // 44
} TaskRegisters;

typedef struct Task {
    TaskRegisters registers;
    uint32_t* pageDirectory;

    struct Task* prev;
    struct Task* next;
} Task;

extern Task* CurrentTask;

Task* TaskInit(void);
void TaskFree(Task* task);
void SetCurrentTask(Task* task);

int CopyPageFromTask(Task* task, void* dest, const void* va);
uint32_t GetStackElement(Task* task, int index);

extern void GoToUserMode(TaskRegisters* registers);
extern void SetSegmentRegistersToKernel(void);
extern void SetSegmentRegistersToUser(void);
extern void SetRegisters(TaskRegisters* registers);
