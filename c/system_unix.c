#define _POSIX_C_SOURCE 199309L // minimum for clock_gettime()

#include <stdio.h>
#include <inttypes.h>
#include <stddef.h>
#include <time.h>
#include <sys/time.h>
#include <sys/resource.h>

#undef NDEBUG
#include <assert.h>

#include "system.h"

void system_print_memstats(void) {
#ifdef FOO_MACOS
  return;
#else
  FILE* f = fopen("/proc/self/statm", "r");
  if (!f) {
    fprintf(stderr, "Could not open /proc/self/statm\n");
  } else {
    int c;
    while (EOF != (c = fgetc(f))) {
      fputc(c, stderr);
    }
    fflush(stderr);
  }
#endif
}

bool system_is_unix(void) {
  return true;
}

bool system_is_macos(void) {
#ifdef FOO_MACOS
  return true;
#else
  return false;
#endif
}

bool system_is_windows(void) {
  return false;
}

double timeval_as_double(struct timeval t) {
  return (double)t.tv_sec + (double)t.tv_usec / 1e6;
}

double timespec_as_double(struct timespec t) {
  return (double)t.tv_sec + (double)t.tv_nsec / 1e9;
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
  assert(!clock_gettime(CLOCK_MONOTONIC, &now));
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

int64_t system_random(void) {
  int64_t r;
  FILE* f;
  // Yes, /dev/urandom, not /dev/random. See:
  //     https://www.2uo.de/myths-about-urandom/
  // ...just hoping this is good for other unix
  // platforms as well.
  assert((f = fopen("/dev/urandom", "r")));
  assert(sizeof(r) == fread(&r, 1, sizeof(r), f));
  fclose(f);
  return r;
}

void system_init(void) {
  // Init time
  struct timespec now;
  assert(!clock_gettime(CLOCK_MONOTONIC, &now));
  SYSTEM_START_MONOTONIC_SECONDS = timespec_as_double(now);
}
