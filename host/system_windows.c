#include "system.h"

#define WIN32_LEAN_AND_MEAN
#include <Windows.h>
#include <stdint.h>
#include <winsock2.h>

double system_time_seconds(void) {
  // Matches the Unix epoch!
  //
  // Adapted from: https://stackoverflow.com/questions/10905892/equivalent-of-gettimeday-for-windows
  //
  // Note: some broken versions only have 8 trailing zero's, the correct epoch
  // has 9 trailing zero's This magic number is the number of 100 nanosecond
  // intervals since January 1, 1601 (UTC) until 00:00:00 January 1, 1970
  static const uint64_t EPOCH = ((uint64_t) 116444736000000000ULL);

  SYSTEMTIME  system_time;
  FILETIME    file_time;
  uint64_t    time;

  GetSystemTime(&system_time);
  SystemTimeToFileTime(&system_time, &file_time);
  time =  (uint64_t)file_time.dwLowDateTime;
  time += (uint64_t)file_time.dwHighDateTime << 32;

  return (time - EPOCH) / 10000000.0 + system_time.wMilliseconds / 1000.0;
}
