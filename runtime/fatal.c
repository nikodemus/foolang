#include "fatal.h"

#include <stdlib.h>
#include <stdio.h>
#include <stdarg.h>

void fatal(const char* fmt, ...) {
  va_list arguments;
  va_start(arguments, fmt);
  printf("FATAL: ");
  vprintf(fmt, arguments);
  printf("\n");
  _Exit(1);
}
