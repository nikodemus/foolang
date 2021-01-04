#include <inttypes.h>
#include <stddef.h>
#include <sys/time.h>
#include <sys/resource.h>

#include "system.h"

double timeval_as_double(struct timeval t) {
  return (double)t.tv_sec + (double)(t.tv_usec / 1e6);
}

struct timespec double_as_timespec(double t) {
  if (t < 0.0) {
    t = 0.0;
  }
  time_t i = t;
  return (struct timespec)
    { .tv_sec = i,
      .tv_nsec = (long)((t - i) * 1e9) };
}

double system_time_seconds(void) {
  struct timeval t;
  gettimeofday(&t, NULL);
  return timeval_as_double(t);
}

double SYSTEM_START_MONOTONIC_SECONDS = 0.0;

void system_get_process_times(struct FooProcessTimes* times) {
  struct rusage usage;
  struct timespec now;
  assert(!getrusage(RUSAGE_SELF, &usage));
  assert(!clock_getttime(CLOCK_MONOTONIC, &now));
  times->user = timeval_as_double(usage.ru_utime);
  times->system = timeval_as_double(usage.ru_stime);
  times->real = timespec_as_double(now) - SYSTEM_START_MONOTONIC_SECONDS;
}

void system_sleep(double seconds) {
  struct timespec req = double_as_timespec(seconds);
  struct timespec rem_unused;
  // FIXME: Resume sleep after interrupt?
  nanosleep(&req, &rem_unused);
}

void system_init() {
  // Init time
  struct timespec now;
  assert(!clock_getttime(CLOCK_MONOTONIC, &now));
  SYSTEM_START_MONOTONIC_SECONDS = timespec_as_double(now);
}
