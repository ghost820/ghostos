#include "task.h"

#include "heap.h"
#include "memory.h"
#include "paging.h"

Task* firstTask;
Task* CurrentTask;
Task* lastTask;

Task*
TaskInit(void)
{
    Task* task = (Task*)kzalloc(sizeof(Task));

    task->registers.eip = 0x400000;
    task->registers.esp = 0x3ff000;
    task->registers.cs = 0x1b;
    task->registers.ss = 0x23;

    task->pageDirectory = CreatePageDirectory(
        PAGING_PRESENT | PAGING_USER_SUPERVISOR | PAGING_READWRITE,
        PAGING_PRESENT | PAGING_USER_SUPERVISOR);

    if (!firstTask) {
        firstTask = task;
        lastTask = task;
        return task;
    }

    lastTask->next = task;
    task->prev = lastTask;
    lastTask = task;

    return task;
}

void
TaskFree(Task* task)
{
    FreePageDirectory(task->pageDirectory);

    if (task->prev) {
        task->prev->next = task->next;
    }

    if (task->next) {
        task->next->prev = task->prev;
    }

    if (task == firstTask) {
        firstTask = task->next;
    }

    if (task == lastTask) {
        lastTask = task->prev;
    }

    if (task == CurrentTask) {
        CurrentTask = 0;
    }

    kfree(task);
}

void
SetCurrentTask(Task* task)
{
    // TODO: Check if this should be here
    // SetSegmentRegistersToUser();
    SetPageDirectory(task->pageDirectory);
    CurrentTask = task;
    GoToUserMode(&task->registers);
}

int
CopyPageFromTask(Task* task, void* dest, const void* va)
{
    void* buffer = kmalloc(4096);
    if (!buffer) {
        return -1;
    }
    if (buffer == va) {
        kfree(buffer);
        return -2;
    }

    PageTableEntry entry = GetPageMapping(task->pageDirectory, buffer);
    SetPageMapping(task->pageDirectory, buffer, buffer, PAGING_PRESENT | PAGING_READWRITE);
    SetPageDirectory(task->pageDirectory);
    memcpy(buffer, va, 4096);
    SetPageDirectory(PAGE_DIRECTORY_KERNEL);
    SetPageMapping(task->pageDirectory, buffer, entry.pa, entry.flags);

    memcpy(dest, buffer, 4096);

    kfree(buffer);
    return 0;
}

uint32_t
GetStackElement(Task* task, int index)
{
    uint32_t result = 0;
    uint32_t* stack = (uint32_t*)task->registers.esp;

    SetSegmentRegistersToUser();
    SetPageDirectory(task->pageDirectory);

    result = stack[index];

    SetSegmentRegistersToKernel();
    SetPageDirectory(PAGE_DIRECTORY_KERNEL);

    return result;
}
