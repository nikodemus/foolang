#include "utils.h"

#include <stdlib.h>
#include <stdio.h>
#include <stdarg.h>

size_t minz(size_t a, size_t b) {
  if (a < b)
    return a;
  else
    return b;
}

void fatal(const char* fmt, ...) {
  va_list arguments;
  va_start(arguments, fmt);
  printf("FATAL: ");
  vprintf(fmt, arguments);
  printf("\n");
  _Exit(1);
}
