#include "process.h"

#include "memory.h"
#include "heap.h"
#include "paging.h"

Process* processes[MAX_PROCESSES];
Process* currentProcess;

void ProcessEnvironmentInit(void) {
    memset(processes, 0, sizeof(processes));
    currentProcess = 0;
}

Process* ProcessInit(const void* data, uint32_t dataSize) {
    Process* process = kzalloc(sizeof(Process));
    if (!process) {
        return 0;
    }

    process->data = kmalloc(dataSize);
    if (!process->data) {
        kfree(process);
        return 0;
    }
    memcpy(process->data, data, dataSize);
    process->dataSize = dataSize;

    int i;
    for (i = 0; i < MAX_PROCESSES; i++) {
        if (processes[i] == 0) {
            processes[i] = process;
            process->pid = i;
            break;
        }
    }
    if (i == MAX_PROCESSES) {
        kfree(process->data);
        kfree(process);
        return 0;
    }

    process->stack = kmalloc(PROCESS_STACK_SIZE);
    if (!process->stack) {
        processes[i] = 0;
        kfree(process->data);
        kfree(process);
        return 0;
    }

    process->mainThread = TaskInit();
    if (!process->mainThread) {
        processes[i] = 0;
        kfree(process->stack);
        kfree(process->data);
        kfree(process);
        return 0;
    }

    uint8_t* va = (uint8_t*)0x400000;
    uint8_t* pa = (uint8_t*)process->data;
    for (uint32_t i = 0; i < BytesToPages(process->dataSize); i++) {
        SetPageMapping(va, pa, PAGING_PRESENT | PAGING_USER_SUPERVISOR | PAGING_READWRITE);
        va += 4096;
        pa += 4096;
    }

    return process;
}
