#ifndef __SYSTEM_H_
#define __SYSTEM_H_

#include <stdint.h>

#include "foo.h"

double system_time_seconds(void);
void system_get_process_times(struct FooProcessTimes* times);
void system_sleep(double seconds);
int64_t system_random();
void system_init();

#endif // __SYSTEM_H_
