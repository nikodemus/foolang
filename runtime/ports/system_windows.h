#ifndef __SYSTEM_WINDOWS_H_
#define __SYSTEM_WINDOWS_H_

#define WIN32_LEAN_AND_MEAN
#include <Windows.h>

typedef CRITICAL_SECTION* SystemLock_t;
typedef HANDLE SystemThread_t;

#endif // __SYSTEM_WINDOWS_H_
