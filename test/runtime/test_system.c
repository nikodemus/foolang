#define TEST_NO_MAIN
#define _CRT_SECURE_NO_WARNINGS 1

#include "foolang.h"
#include "acutest/acutest.h"

#include <stdio.h>

void test_system_number_of_cpu_cores(void) {
    TEST_CHECK(system_number_of_cpu_cores() > 1);
}

SystemLock_t SystemLock_test_lock;
int SystemLock_test_state = 0;

void SystemLock_test_function(void* data) {
    (void)data;
    system_lock(SystemLock_test_lock);
    TEST_CHECK(SystemLock_test_state == 1);
    SystemLock_test_state = 2;
    system_unlock(SystemLock_test_lock);
}

void test_SystemLock(void) {
    SystemLock_test_lock = make_SystemLock();
    system_lock(SystemLock_test_lock);
    SystemThread_t thread = make_SystemThread(SystemLock_test_function, NULL);
    system_sleep_ms(1);
    SystemLock_test_state = 1;
    system_unlock(SystemLock_test_lock);
    system_join_thread(thread);
    TEST_CHECK(SystemLock_test_state == 2);
    free_SystemLock(SystemLock_test_lock);
}

size_t SystemThread_test_var = 0;
const size_t SystemThread_test_incs = 100000;

void SystemThread_test_function(void* data) {
    _Atomic size_t* ptr = data;
    for (size_t i = 0; i < SystemThread_test_incs; i++)
        (*ptr)++;
}

void test_SystemThread(void) {
    size_t test_size = 10;
    SystemThread_t thread[test_size];
    for (size_t i = 0; i < test_size; i++)
        thread[i] = make_SystemThread(SystemThread_test_function,
				      &SystemThread_test_var);
    for (size_t i = 0; i < test_size; i++)
        TEST_ASSERT(system_join_thread(thread[i]));
    TEST_CHECK_(SystemThread_test_var == SystemThread_test_incs * test_size, "var = %zu", SystemThread_test_var);
}
