#define TEST_NO_MAIN
#define _CRT_SECURE_NO_WARNINGS 1

#include "foo.h"
#include "ext/acutest.h"

#include <stdio.h>

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
