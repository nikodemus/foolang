#define TEST_NO_MAIN
#define _CRT_SECURE_NO_WARNINGS 1

#include "foo.h"
#include "ext/acutest.h"

#include <stdatomic.h>
#include <stdio.h>

void test_make_executor_pool(void) {
  const size_t n = 17;
  struct ExecutorPool* pool = make_ExecutorPool(n);
  TEST_ASSERT(pool->size == n);
  for (size_t i = 0; i < n; i++) {
    struct Executor* executor = pool->executors[i];
    TEST_ASSERT(executor->id == i);
    TEST_ASSERT(executor_get_state(executor) == ExecutorSuspended);
  }
  free_ExecutorPool(pool);
}

struct TestData {
  SystemLock_t lock;
  int state;
};

char* test_actor_entry(char* sp, struct Actor* actor) {
  (void)sp;
  struct TestData* data = actor->data;
  system_lock(data->lock);
  TEST_CHECK(data->state == 1);
  data->state = 2;
  system_unlock(data->lock);
  // Since we're yielding we need to set our own sp.
  actor->state = ActorExiting;
  actor->sp = sp;
  return NULL;
}

void test_executors_run(void) {
  const size_t n = 2;
  struct ExecutorPool* pool = make_ExecutorPool(n);
  // start_pool(pool);
  for (size_t i = 0; i < n; i++) {
    struct Executor* executor = pool->executors[i];
    TEST_ASSERT(atomic_load(&executor->state) == ExecutorSuspended);
  }
  start_pool(pool);
  system_sleep_ms(10);
  struct TestData data = {
    .lock = make_SystemLock(),
    .state = 0
  };
  system_lock(data.lock);
  struct Actor* actor = make_Actor(test_actor_entry, &data);
  enqueue_actor(pool->executors[0]->queue, actor);
  system_sleep_ms(1);
  TEST_CHECK(data.state == 0);
  data.state = 1;
  system_unlock(data.lock);
  wait_for_actor_exit(actor, 10);
  TEST_CHECK(data.state == 2);
  free_Actor(actor);
  free_SystemLock(data.lock);
  stop_pool(pool);
  free_ExecutorPool(pool);
}

void test_ExecutorPool(void) {
  test_make_executor_pool();
  test_executors_run();
}
