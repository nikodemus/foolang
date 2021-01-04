#ifndef __SYSTEM_H_
#define __SYSTEM_H_

#include "foo.h"

double system_time_seconds(void);
void system_get_process_times(struct FooProcessTimes* times);
void system_sleep(double seconds);
void system_init();

#endif // __SYSTEM_H_
