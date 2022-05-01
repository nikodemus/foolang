#ifndef __EXECUTOR_H_
#define __EXECUTOR_H_

#include "actor_queue.h"
#include "system.h"

enum ExecutorState {
  ExecutorInterrupt,
  ExecutorRunning,
  ExecutorSuspended,
};

struct ExecutorPool;

struct Executor {
  size_t id;
  struct ExecutorPool* pool;
  _Atomic enum ExecutorState state;
  SystemLock_t lock;
  SystemThread_t thread;
  struct ActorQueue* queue;
};

struct Executor* make_Executor(size_t id, struct ExecutorPool*);
void start_executor(struct Executor*);
void stop_executor(struct Executor*);
void free_Executor(struct Executor*);
enum ExecutorState executor_get_state(struct Executor* executor);

#endif // __EXECUTOR_H_
