#ifndef __EXECUTOR_H_
#define __EXECUTOR_H_

#include "actor_queue.h"
#include "system.h"

enum ExecutorState {
  ExecutorIdle,
  ExecutorRunning,
};

struct Executor {
  size_t id;
  _Atomic enum ExecutorState state;
  SystemLock_t lock;
  struct ActorQueue* queue;
};

struct Executor* make_Executor(size_t id);
void free_Executor(struct Executor*);
enum ExecutorState executor_get_state(struct Executor* executor);

#endif // __EXECUTOR_H_
