#ifndef __SYSTEM_H_
#define __SYSTEM_H_

#include <stdint.h>
#include <stdbool.h>

#include "foo.h"

void* system_input(void);
bool system_input_at_eof(FILE* input);
int system_input_read_char(FILE* input);
bool system_input_unread_char(FILE* input, int c);

void system_exit(int)  __attribute__ ((noreturn));
bool system_is_unix(void);
bool system_is_windows(void);
bool system_is_macos(void);
double system_time_seconds(void);
void system_get_process_times(struct FooProcessTimes* times);
void system_sleep(double seconds);
int64_t system_random();
void system_init();

#endif // __SYSTEM_H_
