#ifndef __SYSTEM_H_
#define __SYSTEM_H_

#ifdef _WIN32
#include "system_windows.h"
#endif

#include <stdbool.h>

size_t system_number_of_cpu_cores();

SystemLock_t make_SystemLock();
void free_SystemLock(SystemLock_t lock);
void system_lock(SystemLock_t lock);
void system_unlock(SystemLock_t lock);

#endif // __SYSTEM_H_
