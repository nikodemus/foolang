#define _CRT_SECURE_NO_WARNINGS 1

#include "foo.h"
#include "ext/acutest.h"

void test_system_number_of_cpu_cores(void) {
    TEST_CHECK(system_number_of_cpu_cores() > 1);
}

void test_make_ExecutorPool(void) {
    const size_t n = 17;
    struct ExecutorPool* pool = make_ExecutorPool(n);
    TEST_ASSERT(pool->size == n);
    for (size_t i = 0; i < n; i++) {
        struct Executor* executor = pool->executors[i];
        TEST_ASSERT(executor->id == i);
        TEST_ASSERT(executor_get_state(executor) == ExecutorIdle);
    }
}

void test_SystemLock() {
    SystemLock_t mylock = make_SystemLock();
    system_lock(mylock);
    system_unlock(mylock);
}

#define DO(name) { #name, test_ ## name }

TEST_LIST = {
    DO(SystemLock),
    DO(make_ExecutorPool),
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
