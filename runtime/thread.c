#include "thread.h"

#include <stdlib.h>

#include "fatal.h"

struct ThreadInfo* make_ThreadInfo(ThreadFunction function, void* parameter) {
  struct ThreadInfo* info = malloc(sizeof(struct ThreadInfo));
  if (!info)
    fatal("Could not allocate memory for thread information");
  info->function = function;
  info->parameter = parameter;
  return info;
}

void free_ThreadInfo(struct ThreadInfo* info) {
  free(info);
}
