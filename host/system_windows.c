#include "system.h"
#include <Windows.h>
#include <inttypes.h>

union FileTime {
  FILETIME as_struct;
  uint64_t as_int;
};

double system_time_seconds(void) {
  union FileTime t;
  GetSystemTimeAsFileTime(&t.as_struct);
  return t.as_int / 1e7;
}
