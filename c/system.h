#ifndef __SYSTEM_H_
#define __SYSTEM_H_

#include <stdint.h>
#include <stdbool.h>

#include "foo.h"

void* system_input(void);
bool  system_input_at_eof(void* input);
bool  system_input_get_buffering(struct FooContext* sender, void* input);
bool  system_input_get_echo(struct FooContext* sender, void* input);
bool  system_input_set_buffering(struct FooContext* sender, void* input, bool echo);
bool  system_input_set_echo(struct FooContext* sender, void* input, bool buffering);
bool  system_input_unread_char(void* input, int c);
int   system_input_read_char(void* input);
int   system_input_read_char_timeout(struct FooContext* sender, void* input, double seconds);

void* system_output(void);
void  system_output_flush(struct FooContext* sender, void* output);
void  system_output_write_bytes(struct FooContext* sender, void* output, struct FooBytes* bytes);
bool  system_output_set_processed(struct FooContext* sender, void* output, bool processed);

void* system_filestream_as_input_ptr(struct FooContext* sender, void* filestream);
void* system_filestream_as_output_ptr(struct FooContext* sender, void* filestream);

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
