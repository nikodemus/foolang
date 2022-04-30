#include "executor_pool.h"

#include "executor.h"
#include "fatal.h"

#include <stdlib.h>

struct ExecutorPool* make_ExecutorPool(size_t size) {
  struct ExecutorPool* pool = malloc(sizeof(struct ExecutorPool));
  if (!pool) {
    fatal("Could not allocate executor pool. Size: %zu", size);
  }
  pool->size = size;
  pool->executors = malloc(size * sizeof(struct Executor**));
  if (!pool->executors) {
    fatal("Count not allocate executor pool array. Size: %zu", size);
  }
  for (size_t i = 0; i < size; i++) {
    pool->executors[i] = make_Executor(i);
  }
  return pool;
}
