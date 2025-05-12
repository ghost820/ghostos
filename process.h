#pragma once

#include "types.h"

#include "task.h"

#define MAX_PROCESSES 256
#define PROCESS_STACK_SIZE 16384

typedef struct {
    uint16_t pid;

    Task* mainThread;

    void* stack;

    void* data;
    uint32_t dataSize;
} Process;

void ProcessEnvironmentInit(void);
Process* ProcessInit(const void* data, uint32_t dataSize);
