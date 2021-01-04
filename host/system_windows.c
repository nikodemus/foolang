#include "system.h"

#define WIN32_LEAN_AND_MEAN
#include <Windows.h>
#include <stdint.h>
#include <winsock2.h>

#undef NDEBUG
#include <assert.h>
#include <stdio.h>

uint64_t filetime_as_u64(FILETIME t) {
  return (uint64_t)t.dwHighDateTime << 32 | (uint64_t)t.dwLowDateTime;
}

FILETIME u64_as_filetime(uint64_t t) {
  FILETIME f;
  f.dwLowDateTime = (uint32_t)(t & 0xffffffff);
  f.dwHighDateTime = (uint32_t)(t >> 32);
  return f;
}

double u64_as_double(uint64_t t) {
  // Input is a FILETIME converted into a 64-bit value, so it represents
  // 100-nanosecond intervals.
  //
  // Divide by 10 gives us microseconds, which fit into a double "well enough".
  // FIXME: How well? Proof here.
  uint64_t microseconds = t / 10;
  return (double)microseconds / 1e6;
}

double filetime_as_double(FILETIME t) {
  return u64_as_double(filetime_as_u64(t));
}

double system_time_seconds(void) {
  // This magic number is the number of 100 nanosecond intervals since January
  // 1, 1601 (UTC) until 00:00:00 January 1, 1970, allowing us to go from
  // Windows to Unix epoch.
  static const uint64_t EPOCH = 116444736000000000ULL;

  FILETIME now0;
  GetSystemTimePreciseAsFileTime(&now0);

  uint64_t now = filetime_as_u64(now0);
  return u64_as_double(now - EPOCH);
}

void system_get_process_times(struct FooProcessTimes* times) {
  HANDLE proc = GetCurrentProcess();
  FILETIME created, exited_unused, kernel, user, now;
  assert(GetProcessTimes(proc, &created, &exited_unused, &kernel, &user));
  GetSystemTimePreciseAsFileTime(&now);
  fflush(stdout);
  uint64_t real = filetime_as_u64(now) - filetime_as_u64(created);
  times->system = filetime_as_double(kernel);
  times->user = filetime_as_double(user);
  times->real = u64_as_double(real);
}

void system_sleep(double seconds) {
  if (seconds < 0.0) {
    seconds = 0.0;
  }
  // Windows sleeps in milliseconds
  Sleep((DWORD)(seconds * 1000.0));
}

void system_init(void) {
}
