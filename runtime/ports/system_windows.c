#include "system_windows.h"

#include "config.h"
#include "utils.h"

#include <stdio.h>
#include <stdlib.h>
#include <sysinfoapi.h>

size_t system_number_of_cpu_cores(void) {
    SYSTEM_INFO system_info;
    GetSystemInfo(&system_info);
    return (size_t)system_info.dwNumberOfProcessors;
}

SystemLock_t make_SystemLock(void) {
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

struct ThreadInfo {
    ThreadFunction function;
    void* parameter;
};

DWORD WINAPI run_thread(void* data) {
    struct ThreadInfo* info = data;
    ThreadFunction function = info->function;
    void* parameter = info->parameter;
    free(info);
    function(parameter);
    return 0;
}

typedef LPTHREAD_START_ROUTINE SystemThreadFunction_t;

SystemThread_t make_SystemThread(ThreadFunction function, void* parameter) {
    struct ThreadInfo* info = malloc(sizeof(struct ThreadInfo));
    info->function = function;
    info->parameter = parameter;
    return CreateThread(NULL,  // cannot be inherited by child processes
                        0,     // default stack size
                        run_thread,
                        (void*)info,
                        0,     // run immediately after creation
                        NULL); // we don't need the thread id
}

bool system_join_thread(SystemThread_t thread) {
  if (WAIT_OBJECT_0 == WaitForSingleObject(thread, INFINITE)) {
    CloseHandle(thread);
    return true;
  } else {
    return false;
  }
}

void system_sleep_ms(size_t ms) {
    Sleep((DWORD)ms);
}
