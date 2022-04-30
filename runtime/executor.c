#include "executor.h"

#include "fatal.h"
#include "system.h"

#include <stdatomic.h>
#include <stdlib.h>

struct Executor* make_Executor(size_t id) {
  struct Executor* executor = malloc(sizeof(struct Executor));
  if (!executor) {
    fatal("Could not allocate executor. Id: %zu", id);
  }
  executor->id = id;
  atomic_init(&executor->state, ExecutorIdle);
  executor->lock = make_SystemLock();
  return executor;
}


void free_Executor(struct Executor* executor) {
  free_SystemLock(executor->lock);
  free(executor);
}

enum ExecutorState executor_get_state(struct Executor* executor) {
  return atomic_load(&executor->state);
}
