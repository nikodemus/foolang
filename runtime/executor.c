#include "executor.h"

#include "actor.h"
#include "actor_queue.h"
#include "config.h"
#include "executor_pool.h"
#include "system.h"
#include "utils.h"

#include <stdatomic.h>
#include <stdlib.h>
#include <stdio.h>

#undef NDEBUG
#include <assert.h>

struct Executor* make_Executor(size_t id, struct ExecutorPool* pool) {
  struct Executor* executor = malloc(sizeof(struct Executor));
  if (!executor) {
    fatal("Could not allocate executor. Id: %zu", id);
  }
  executor->id = id;
  executor->pool = pool;
  atomic_init(&executor->state, ExecutorSuspended);
  executor->lock = make_SystemLock();
  executor->queue = make_ActorQueue();
  return executor;
}

void run_executor(struct Executor* executor, struct Actor* actor) {
  size_t run = FOO_EXECUTOR_INTERRUPT_DELAY;
  while (actor && run--) {
    assert(ActorReady == actor->state);
    enum ActorState state = run_actor_timeslice(actor);
    if (state == ActorReady) {
      // If queue is empty, don't requeue, just keep it:
      // requeueing would give another the opportunity to
      // steal it, which doesn't make much sense.
      if (atomic_load(&executor->queue->size) == 0) {
        continue;
      } else {
        enqueue_actor(executor->queue, actor);
      }
    } else if (state == ActorExiting) {
      // Nothing to do.
      ;
    } else {
      fatal("Bad actor state after timeslice: %zu", (size_t)state);
    }
    actor = dequeue_actor(executor->queue);
  }
}

struct Actor* steal_actor(struct Executor* executor) {
  for (size_t id = executor->id; id < executor->pool->size; id++) {
    struct Actor* actor = dequeue_actor(executor->pool->executors[id]->queue);
    if (actor)
      return actor;
  }
  for (size_t id = 0; id <= executor->id; id++) {
    struct Actor* actor = dequeue_actor(executor->pool->executors[id]->queue);
    if (actor)
      return actor;

  }
  return NULL;
}

void run_executor_loop(void* executor0) {
  struct Executor* executor = executor0;
  atomic_store(&executor->state, ExecutorRunning);
  while (atomic_load(&executor->state) == ExecutorRunning) {
    struct Actor* actor = steal_actor(executor);
    if (actor)
      run_executor(executor, actor);
    else
      system_sleep_ms(1);
  }
}

void start_executor(struct Executor* executor) {
  executor->thread = make_SystemThread(run_executor_loop, executor);
}

void stop_executor(struct Executor* executor) {
  atomic_store(&executor->state, ExecutorInterrupt);
  system_join_thread(executor->thread);
  atomic_store(&executor->state, ExecutorSuspended);
}

void free_Executor(struct Executor* executor) {
  free_ActorQueue(executor->queue);
  free_SystemLock(executor->lock);
  free(executor);
}

enum ExecutorState executor_get_state(struct Executor* executor) {
  return atomic_load(&executor->state);
}
