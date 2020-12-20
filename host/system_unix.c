#include "system.h"
#include <inttypes.h>
#include <stddef.h>
#include <sys/time.h>

double system_time_seconds(void) {
  struct timeval t;
  gettimeofday(&t, NULL);
  return (double)t.tv_sec + (double)(t.tv_usec / 1e6);
}
