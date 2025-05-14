#include "task.h"

#include "heap.h"
#include "paging.h"

Task* firstTask;
Task* currentTask;
Task* lastTask;

Task* TaskInit(void) {
    Task* task = (Task*)kzalloc(sizeof(Task));

    task->registers.eip = 0x400000;
    task->registers.esp = 0x3ff000;
    task->registers.ss = 0x23;

    task->pageDirectory = CreatePageDirectory(
        PAGING_PRESENT | PAGING_USER_SUPERVISOR | PAGING_READWRITE,
        PAGING_PRESENT | PAGING_USER_SUPERVISOR
    );

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

void TaskFree(Task* task) {
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

    if (task == currentTask) {
        currentTask = 0;
    }

    kfree(task);
}

void SetCurrentTask(Task* task) {
    // TODO: Check if this should be here
    SetSegmentRegistersToUser();
    SetPageDirectory(task->pageDirectory);
    currentTask = task;
}
