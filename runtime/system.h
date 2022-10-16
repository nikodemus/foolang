#ifndef __SYSTEM_H_
#define __SYSTEM_H_

#ifdef _WIN32
#include "ports/system_windows.h"
#else
#include "ports/system_posix.h"
#endif

#include "thread_info.h"

#include <stdbool.h>

size_t system_number_of_cpu_cores(void);

SystemLock_t make_SystemLock(void);
void free_SystemLock(SystemLock_t);
void system_lock(SystemLock_t);
void system_unlock(SystemLock_t);

SystemThread_t make_SystemThread(struct ThreadInfo*);
bool system_join_thread(SystemThread_t);

void system_sleep_ms(size_t);

#endif // __SYSTEM_H_
