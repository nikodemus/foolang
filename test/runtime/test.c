#define _CRT_SECURE_NO_WARNINGS 1

#include "foolang.h"
#include "acutest/acutest.h"

void test_ActorQueue_enqueue_and_dequeue(void);
void test_ActorQueue_enqueue_at_capacity(void);

void test_ExecutorPool_make_executor_pool(void);
void test_ExecutorPool_executors_run(void);

void test_SystemThread(void);
void test_SystemLock(void);
void test_system_number_of_cpu_cores(void);

#define DO(name) { #name, test_ ## name }

TEST_LIST = {
    // actor_queue.c
    DO(ActorQueue_enqueue_and_dequeue),
    DO(ActorQueue_enqueue_at_capacity),
    // executor_pool.c
    DO(ExecutorPool_make_executor_pool),
    DO(ExecutorPool_executors_run),
    // system.c
    DO(SystemLock),
    DO(SystemThread),
    DO(system_number_of_cpu_cores),
    { NULL, NULL }
};
