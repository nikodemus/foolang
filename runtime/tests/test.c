#define _CRT_SECURE_NO_WARNINGS 1

#include "foo.h"
#include "ext/acutest.h"

#include <stdio.h>

void test_system_number_of_cpu_cores(void) {
    TEST_CHECK(system_number_of_cpu_cores() > 1);
}

SystemLock_t test_SystemLock_lock;
int test_SystemLock_state = 0;

void test_SystemLock_function(void* data) {
    (void)data;
    system_lock(test_SystemLock_lock);
    TEST_CHECK(test_SystemLock_state == 1);
    test_SystemLock_state = 2;
    system_unlock(test_SystemLock_lock);
}

void test_SystemLock() {
    struct ThreadInfo* info = make_ThreadInfo(test_SystemLock_function, NULL);
    test_SystemLock_lock = make_SystemLock();
    system_lock(test_SystemLock_lock);
    SystemThread_t thread = make_SystemThread(info);
    system_sleep_ms(1);
    test_SystemLock_state = 1;
    system_unlock(test_SystemLock_lock);
    system_join_thread(thread);
    TEST_CHECK(test_SystemLock_state == 2);
    free_SystemLock(test_SystemLock_lock);
}

size_t test_SystemThread_var = 0;
const size_t test_SystemThread_incs = 100000;

void test_SystemThread_function(void* data) {
    _Atomic size_t* ptr = data;
    for (size_t i = 0; i < test_SystemThread_incs; i++)
        (*ptr)++;
}

void test_SystemThread() {
    struct ThreadInfo* info = make_ThreadInfo(test_SystemThread_function,
                                              &test_SystemThread_var);
    size_t test_size = 10;
    SystemThread_t thread[test_size];
    for (size_t i = 0; i < test_size; i++)
        thread[i] = make_SystemThread(info);
    for (size_t i = 0; i < test_size; i++)
        TEST_ASSERT(system_join_thread(thread[i]));
    free_ThreadInfo(info);
    TEST_CHECK_(test_SystemThread_var == test_SystemThread_incs * test_size, "var = %zu", test_SystemThread_var);
}

void test_ActorQueue();
void test_ExecutorPool();

#define DO(name) { #name, test_ ## name }

TEST_LIST = {
    DO(ActorQueue),
    DO(ExecutorPool),
    DO(SystemLock),
    DO(SystemThread),
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
