#include "system.h"

#include "config.h"
#include "fatal.h"
#include "thread_info.h"

#include <stdio.h>
#include <stdlib.h>
#include <sysinfoapi.h>

size_t system_number_of_cpu_cores() {
    SYSTEM_INFO system_info;
    GetSystemInfo(&system_info);
    return (size_t)system_info.dwNumberOfProcessors;
}

SystemLock_t make_SystemLock() {
    CRITICAL_SECTION* critical_section = malloc(sizeof(CRITICAL_SECTION));
    if (!critical_section) {
        fatal("Coult no allocate memory for a system lock.");
    }
    InitializeCriticalSectionAndSpinCount
        (critical_section, FOO_SYSTEM_LOCK_SPIN_COUNT);
    return critical_section;
}

void free_SystemLock(SystemLock_t lock) {
    DeleteCriticalSection(lock);
    free(lock);
}

void system_lock(SystemLock_t lock) {
    EnterCriticalSection(lock);
}

void system_unlock(SystemLock_t lock) {
    LeaveCriticalSection(lock);
}

DWORD WINAPI run_thread(void* data) {
    struct ThreadInfo* info = data;
    info->function(info->parameter);
    return 0;
}

typedef LPTHREAD_START_ROUTINE SystemThreadFunction_t;

SystemThread_t make_SystemThread(struct ThreadInfo* info) {
    return CreateThread(NULL,  // cannot be inherited by child processes
                        0,     // default stack size
                        run_thread,
                        (void*)info,
                        0,     // run immediately after creation
                        NULL); // we don't need the thread id
}

bool system_join_thread(SystemThread_t thread) {
    return (WAIT_OBJECT_0 == WaitForSingleObject(thread, INFINITE));
}

void system_sleep_ms(size_t ms) {
    Sleep((DWORD)ms);
}
