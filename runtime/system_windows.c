#include "system.h"

#include "config.h"
#include "fatal.h"

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
