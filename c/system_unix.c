#define _POSIX_C_SOURCE 199309L // minimum for clock_gettime()

#include <stdlib.h>
#include <stdio.h>
#include <inttypes.h>
#include <stddef.h>
#include <time.h>
#include <sys/time.h>
#include <sys/resource.h>
#include <termios.h>
#include <unistd.h>

#undef NDEBUG
#include <assert.h>

#include "system.h"

void* system_filestream_as_input_ptr(struct FooContext* sender, void* filestream) {
  (void)sender;
  return filestream;
}

void* system_filestream_as_output_ptr(struct FooContext* sender, void* filestream) {
  (void)sender;
  return filestream;
}

void* system_input(void) {
  return stdin;
}

void foo_mark_input(void* ptr) {
  (void)ptr;
  // Just a FILE*, nothing to mark.
}

bool system_input_at_eof(void* input) {
  return 0 != feof(input);
}

int system_input_read_char(void* input) {
  return fgetc(input);
}

bool system_input_unread_char(void* input, int c) {
  return EOF != ungetc(c, input);
}

bool system_set_termios_flags(struct FooContext* sender, FILE* file, int iflag, int oflag, int lflag, bool on) {
  (void)sender;
  int fd = fileno(file);
  struct termios mode;
  tcgetattr(fd, &mode);
  if (on) {
    mode.c_iflag |= iflag;
    mode.c_oflag |= oflag;
    mode.c_lflag |= lflag;
  } else {
    mode.c_iflag &= ~iflag;
    mode.c_oflag &= ~oflag;
    mode.c_lflag &= ~lflag;
  }
  // Should this be TCANOW for input and TCADRAIN for output?
  //
  // TCSANOW: the change occurs immediately.
  //
  // TCSADRAIN: the change occurs after all output written to fd has been
  //    transmitted. This function should be used when changing parameters that
  //    affect output.
  //
  // TCSAFLUSH: the change occurs after all output written to the object referred
  //   by fd has been transmitted, and all input that has been received but not
  //   read will be discarded before the change is made.
  tcsetattr(fd, TCSAFLUSH, &mode);
  return on;
}

bool system_input_set_echo(struct FooContext* sender, void* input, bool echo) {
  return system_set_termios_flags(sender, input, 0, 0, ECHO, echo);
}

bool system_input_set_buffering(struct FooContext* sender, void* input, bool buffering) {
  return system_set_termios_flags(sender, input,
                                  IXON | ICRNL,
                                  0,
                                  ICANON | ISIG | IEXTEN,
                                  buffering);
}

void system_input_get_termios_flags(struct FooContext* sender, FILE* file, int* iflag, int* lflag) {
  (void)sender;
  int fd = fileno(file);
  struct termios mode;
  tcgetattr(fd, &mode);
  if (iflag)
    *iflag = mode.c_iflag;
  if (lflag)
    *lflag = mode.c_lflag;
}

bool system_input_get_echo(struct FooContext* sender, void* input) {
  int lflag;
  system_input_get_termios_flags(sender, input, NULL, &lflag);
  return lflag & ECHO;
}

bool system_input_get_buffering(struct FooContext* sender, void* input) {
  int iflag, lflag;
  system_input_get_termios_flags(sender, input, &iflag, &lflag);
  return (iflag & (IXON | ICRNL)) && (lflag & (ICANON | ISIG | IEXTEN));
}

void* system_output(void) {
  return stdout;
}

void system_output_flush(struct FooContext* sender, void* output) {
  (void)sender;
  fflush(output);
}

void system_output_write_bytes(struct FooContext* sender, void* output, struct FooBytes* bytes) {
  size_t to_write = bytes->size;
  size_t offset = 0;
  while (to_write > 0) {
    size_t wrote = fwrite(bytes->data+offset, 1, to_write, output);
    if (!wrote) {
      foo_panicf(sender, "Could not write to output!");
    }
    to_write -= wrote;
    offset += wrote;
  }
}

bool system_output_set_processed(struct FooContext* sender, void* output, bool processed) {
  return system_set_termios_flags(sender, output,
                                  0,
                                  OPOST,
                                  0,
                                  processed);
}

void system_exit(int code) {
  exit(code);
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

struct termios original_termios;

// In terms of being a completionist it would be better to provide a system
// method for this, but I think this is the right thing 99.99% of the time,
// so I think rather I'll provide System#restoreTerminalOnExit: or similar,
// and do the right thing by default.
void system_restore_termios() {
  tcsetattr(STDIN_FILENO, TCSAFLUSH, &original_termios);
}

void system_init(void) {
  // FIXME: split this into multiple functions

  // Init stdin
  tcgetattr(STDIN_FILENO, &original_termios);
  atexit(system_restore_termios);
  // Init time
  struct timespec now;
  assert(!clock_gettime(CLOCK_MONOTONIC, &now));
  SYSTEM_START_MONOTONIC_SECONDS = timespec_as_double(now);
}
