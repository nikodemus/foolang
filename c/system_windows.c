#define WIN32_LEAN_AND_MEAN
#define _CRT_RAND_S

#include <Windows.h>
#include <stdint.h>
#include <stdlib.h>
#include <winsock2.h>
#include <io.h>
#include <fcntl.h>

#undef NDEBUG
#include <assert.h>
#include <stdio.h>

#include "system.h"
#include "mark-and-sweep.h"

/**
 * So this is a bit silly. I built this on top of HANDLEs because I had trouble
 * getting FILE* -> fd -> HANDLE path to work so that Get/SetConsoleMode would
 * have worked... but it turns out that I had the checking of the return
 * value the wrong way around. So quite plausibly I could get rid of FooInput
 * here...
 *
 * Then again, it does allow supporting Windows specific extensions, so _maybe_
 * now that the work has been done it is worth it?
 */

struct FooInput {
  struct FooHeader header;
  HANDLE handle;
  int buffer;
  bool eof;
};

struct FooInput FooStandardInput = {
  .header = { .allocation = STATIC, .identity_hash = 0 },
  .handle = NULL,
  .buffer = EOF,
  .eof = false
};

void foo_mark_input(void* ptr) {
  struct FooInput* input = ptr;
  if (input->header.allocation == HEAP) {
    foo_mark_live(input);
  }
}

HANDLE FooStandardOutput = 0;

void* system_filestream_as_input_ptr(struct FooContext* sender, void* filestream) {
  struct FooInput* input = foo_alloc(sender, sizeof(struct FooInput));
  input->header.allocation = HEAP;
  input->handle = (HANDLE)_get_osfhandle(_fileno(filestream));
  input->buffer = EOF;
  input->eof = false;
  return input;
}

void* system_filestream_as_output_ptr(struct FooContext* sender, void* filestream) {
  return (void*)_get_osfhandle(_fileno(filestream));
}

void system_oops(const char* what) {
    DWORD code = GetLastError();
    fprintf(stderr, "%s (%zu)\n", what, (size_t)code);
    fflush(stderr);
    _Exit(1);
}

void* system_input(void) {
  return &FooStandardInput;
}

void system_init_input(void) {
  HANDLE input = GetStdHandle(STD_INPUT_HANDLE);
  if (input == INVALID_HANDLE_VALUE) {
    system_oops("ERROR: Could not get STD_INPUT_HANDLE");
  }
  FooStandardInput.handle = input;
  FooStandardInput.buffer = EOF;
  FooStandardInput.eof = false;
}

void system_set_console_mode_bits(struct FooContext* sender, HANDLE console, DWORD bits, bool on) {
  DWORD mode;
  if (!GetConsoleMode(console, &mode)) {
    foo_panicf(sender, "Could not get console mode (%lu)", GetLastError());
  }
  if (on) {
    mode |= bits;
  } else {
    mode &= ~bits;
  }
  if (!SetConsoleMode(console, mode)) {
    foo_panicf(sender, "Could not set console mode (%lu)", GetLastError());
  }
}

bool system_input_set_echo(struct FooContext* sender, void* input, bool echo) {
  system_set_console_mode_bits(sender, ((struct FooInput*)input)->handle, ENABLE_ECHO_INPUT, echo);
  return echo;
}

bool system_input_set_buffering(struct FooContext* sender, void* input, bool buffering) {
  HANDLE console = ((struct FooInput*)input)->handle;
  system_set_console_mode_bits(sender, console, ENABLE_LINE_INPUT, buffering);
  system_set_console_mode_bits(sender, console, ENABLE_PROCESSED_INPUT, buffering);
  system_set_console_mode_bits(sender, console, ENABLE_VIRTUAL_TERMINAL_INPUT, !buffering);
  return buffering;
}

bool system_input_get_mode_bits(struct FooContext* sender, struct FooInput* in, DWORD bits) {
  DWORD mode;
  if (!GetConsoleMode(in->handle, &mode)) {
    foo_panicf(sender, "Could not get console mode (%lu)", GetLastError());
  }
  return mode & bits;
}

bool system_input_get_echo(struct FooContext* sender, void* input) {
  return system_input_get_mode_bits(sender, input, ENABLE_ECHO_INPUT);
}

bool system_input_get_buffering(struct FooContext* sender, void* input) {
  // FIXME: This is a sign of the API being wrong: what if just one is
  // true?
  //
  // Either need to provide more operating specific support, or need
  // to privide an Input#virtualTerminalSequences: or similar, which is
  // then a no-op outside Windows.
  return system_input_get_mode_bits(sender, input,
                                    ENABLE_LINE_INPUT |
                                    ENABLE_PROCESSED_INPUT |
                                    ~ENABLE_VIRTUAL_TERMINAL_INPUT);
}

bool system_input_at_eof(void* input) {
  struct FooInput* in = input;
  return in->eof;
}

int system_input_read_char(void* input) {
  char ch;
  struct FooInput* in = input;

  if (in->buffer != EOF) {
    // There's a buffered character, take it.
    ch = (char)in->buffer;
    in->buffer = EOF;
    return ch;
  }

  // Try to read a single character, set EOF mark on failure.
  DWORD count = 0;
  if (!ReadFile(in->handle, &ch, sizeof(ch), &count, NULL)) {
    system_oops("ERROR: Could not read from input");
  }
  if (count) {
    return ch;
  } else {
    in->eof = true;
    return EOF;
  }
}

int system_input_read_char_timeout(struct FooContext* sender, void* input, double seconds) {
  return system_input_read_char(input);
}

bool system_input_unread_char(void* input, int ch) {
  if (ch == EOF) {
    // This is just to match ungetc() behaviour for sake
    // of consistency.
    return false;
  }
  struct FooInput* in = input;
  if (in->buffer == EOF) {
    in->buffer = ch;
    return true;
  } else {
    return false;
  }
}

void* system_output(void) {
  return (void*)FooStandardOutput;
}

void system_init_output(void) {
  HANDLE output = GetStdHandle(STD_OUTPUT_HANDLE);
  if (output == INVALID_HANDLE_VALUE) {
    system_oops("ERROR: Could not get STD_OUTPUT_HANDLE");
  }
  FooStandardOutput = output;
  // We do this so that all fprintfs from the C-side
  // do what we expect -- mainly backtraces.
  _setmode(_fileno(stdout), O_BINARY);
  _setmode(_fileno(stderr), O_BINARY);
}

void system_output_flush(struct FooContext* sender, void* output) {
  // No buffering at the moment!
  (void)sender;
  (void)output;
}

void system_output_write_bytes(struct FooContext* sender, void* output, struct FooBytes* bytes) {
  (void)sender;
  DWORD to_write = bytes->size;
  DWORD offset = 0;
  while (to_write > 0) {
    DWORD wrote;
    if (!WriteFile((HANDLE)output, bytes->data+offset, to_write, &wrote, NULL)) {
      foo_panicf(sender, "ERROR: Could not write to output! (%lu)", GetLastError());
    }
    to_write -= wrote;
    offset += wrote;
  }
}

bool system_output_set_processed(struct FooContext* sender, void* output, bool processed) {
  HANDLE console = (HANDLE)output;
  system_set_console_mode_bits(sender, console, ENABLE_PROCESSED_OUTPUT, processed);
  system_set_console_mode_bits(sender, console, DISABLE_NEWLINE_AUTO_RETURN, !processed);
  // FIXME: Another sign of the API being wrong...
  //
  // I think I want a more "native" layer, on top of which I can then build
  // a #setupVT or similar.
  system_set_console_mode_bits(sender, console, ENABLE_VIRTUAL_TERMINAL_PROCESSING, !processed);
  return processed;
}

void system_exit(int code) {
  exit(code);
}

bool system_is_unix(void) {
  return false;
}

bool system_is_macos(void) {
  return false;
}

bool system_is_windows(void) {
  return true;
}

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

int64_t system_random(void) {
  // rand_s gives us an unsigned int's worth of random,
  // make sure that's 4 bytes as expected.
  _Static_assert(sizeof(unsigned int) == sizeof(uint32_t),
                 "unsigned int not uint32_t");
  union {
    int64_t value;
    struct{
      uint32_t low;
      uint32_t high;
    };
  } random;
  errno_t err;
  err = rand_s(&random.low);
  assert(!err);
  err = rand_s(&random.high);
  assert(!err);
  return random.value;
}

void system_init(void) {
  system_init_output();
  system_init_input();
}
