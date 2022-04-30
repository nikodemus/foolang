#ifndef __EXECUTOR_POOL_H_
#define __EXECUTOR_POOL_H_

#include "executor.h"

struct ExecutorPool {
  size_t size;
  struct Executor** executors;
};

struct ExecutorPool* make_ExecutorPool(size_t size);
void free_ExecutorPool(struct ExecutorPool*);

#endif // __EXECUTOR_POOL_H_
