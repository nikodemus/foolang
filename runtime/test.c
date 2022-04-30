#define _CRT_SECURE_NO_WARNINGS 1

#include "foo.h"
#include "ext/acutest.h"

#include <stdio.h>

void test_system_number_of_cpu_cores(void) {
    TEST_CHECK(system_number_of_cpu_cores() > 1);
}

void test_ExecutorPool(void) {
    const size_t n = 17;
    struct ExecutorPool* pool = make_ExecutorPool(n);
    TEST_ASSERT(pool->size == n);
    for (size_t i = 0; i < n; i++) {
        struct Executor* executor = pool->executors[i];
        TEST_ASSERT(executor->id == i);
        TEST_ASSERT(executor_get_state(executor) == ExecutorIdle);
    }
    free_ExecutorPool(pool);
}

void test_SystemLock() {
    SystemLock_t mylock = make_SystemLock();
    system_lock(mylock);
    system_unlock(mylock);
    free_SystemLock(mylock);
}

void test_ActorQueue() {
    struct ActorQueue* queue = make_ActorQueue(2);
    // Initialize a test set of 1024 actors.
    const size_t test_size = 1024;
    struct Actor* test_actors[test_size];
    for (size_t i = 0; i < test_size; i++)
        test_actors[i] = make_Actor(i);

    // Enqueue 100
    for (size_t i = 0; i < 100; i++)
        enqueue_actor(queue, test_actors[i]);
    TEST_CHECK(queue_size(queue) == 100);
    TEST_CHECK(queue->start == 0);
    TEST_CHECK(queue->end == 100);

    // Dequeue 10
    for (size_t i = 0; i < 10; i++)
        TEST_CHECK(test_actors[i]->id == dequeue_actor(queue)->id);
    TEST_CHECK(queue_size(queue) == 90);
    TEST_CHECK(queue->start == 10);
    TEST_CHECK(queue->end == 100);

    // Enqueue 100 more
    for (size_t i = 100; i < 200; i++)
        enqueue_actor(queue, test_actors[i]);
    TEST_CHECK_(queue_size(queue) == 190, "size was: %zu", queue_size(queue));

    // Dequeu 10
    for (size_t i = 10; i < 20; i++)
        TEST_CHECK(test_actors[i] == dequeue_actor(queue));
    TEST_CHECK(queue_size(queue) == 180);

    // Enqueue 10 more
    for (size_t i = 200; i < 210; i++)
        enqueue_actor(queue, test_actors[i]);
    TEST_CHECK(queue_size(queue) == 190);

    // Dequeu 100
    for (size_t i = 20; i < 120; i++)
        TEST_CHECK(test_actors[i] == dequeue_actor(queue));
    TEST_CHECK(queue_size(queue) == 90);

    // Empty queue
    while (dequeue_actor(queue))
        ;;
    TEST_CHECK_(queue_size(queue) == 0, "size: %zu", queue_size(queue));
}

#define DO(name) { #name, test_ ## name }

TEST_LIST = {
    DO(ActorQueue),
    DO(SystemLock),
    DO(ExecutorPool),
    DO(system_number_of_cpu_cores),
    { NULL, NULL }
};

/*
int main() {
  printf("core count: %zu\n", n_cores);
  struct ExecutorPool* pool = make_ExecutorPool(n_cores);
  printf("pool size: %zu\n", pool->size);
  for (size_t i = 0; i < pool->size; i++) {
    printf("  executor: %zu\n", pool->executors[i].id);
  }
  return 0;
}
*/
