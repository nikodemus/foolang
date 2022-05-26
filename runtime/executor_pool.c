#include "executor_pool.h"

#include "executor.h"
#include "utils.h"

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
    pool->executors[i] = make_Executor(i, pool);
  }
  return pool;
}

void start_pool(struct ExecutorPool* pool) {
  for (size_t i = 0; i < pool->size; i++) {
    start_executor(pool->executors[i]);
  }
}

void stop_pool(struct ExecutorPool* pool) {
  for (size_t i = 0; i < pool->size; i++) {
    stop_executor(pool->executors[i]);
  }
}

void free_ExecutorPool(struct ExecutorPool* pool) {
  for (size_t i = 0; i < pool->size; i++)
    free_Executor(pool->executors[i]);
  free(pool->executors);
  free(pool);
}
