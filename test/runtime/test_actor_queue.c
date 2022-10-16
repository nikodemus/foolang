#define TEST_NO_MAIN
#define _CRT_SECURE_NO_WARNINGS 1

#include "foolang.h"
#include "acutest/acutest.h"

#include <stdio.h>

void test_ActorQueue_enqueue_and_dequeue(void) {
    struct ActorQueue* queue = make_ActorQueue();
    // Initialize a test set of 1024 actors.
    const size_t test_size = 210;
    struct Actor* test_actors[test_size];
    for (size_t i = 0; i < test_size; i++)
        test_actors[i] = (void*)(intptr_t)i;

    // Enqueue 100
    for (size_t i = 0; i < 100; i++)
        enqueue_actor(queue, test_actors[i]);
    TEST_CHECK(queue_size(queue) == 100);
    TEST_CHECK(queue->start == 0);
    TEST_CHECK(queue->end == 100);

    // Dequeue 10
    for (size_t i = 0; i < 10; i++)
        TEST_CHECK(test_actors[i] == dequeue_actor(queue));
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

    // Enqueue the rest
    for (size_t i = 200; i < test_size; i++)
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

    free_ActorQueue(queue);
}

void test_ActorQueue_enqueue_at_capacity(void) {
    // Setup and sanity check.
    struct ActorQueue* queue = make_ActorQueue();
    intptr_t n = (intptr_t)queue->capacity;
    TEST_ASSERT(n > 1);
    for (intptr_t i = 0; i < n; i++) {
        enqueue_actor(queue, (void*)i);
    }
    TEST_ASSERT(queue->size == (size_t)n);
    TEST_ASSERT(queue->size == queue->capacity);
    TEST_ASSERT(queue->start == 0);
    TEST_ASSERT(queue->end == queue->capacity);
    for (intptr_t i = 0; i < n; i++) {
        TEST_ASSERT(i == (intptr_t)queue->actors[i]);
    }


    // Test growing the queue at capacity with end == capacity.
    for (intptr_t i = n; i < n * 2; i++) {
        enqueue_actor(queue, (void*)i);
    }
    n = n * 2;
    TEST_ASSERT(queue->size == (size_t)n);
    TEST_ASSERT(queue->size == queue->capacity);
    TEST_ASSERT(queue->start == 0);
    TEST_ASSERT(queue->end == queue->capacity);
    for (intptr_t i = 0; i < (intptr_t)queue->size; i++) {
        TEST_CHECK(i == (intptr_t)queue->actors[i]);
    }


    // Test growing the queue at capacity with end == 0
    // (rotate start and end manually).
    intptr_t last = (intptr_t)(queue->actors[queue->end-1]);
    TEST_ASSERT(last == n-1);
    queue->start = queue->capacity - 1;
    queue->end = 0;
    for (intptr_t i = n; i < n * 2; i++) {
        enqueue_actor(queue, (void*)i);
    }
    n = n * 2;
    TEST_ASSERT(queue->size == (size_t)n);
    TEST_ASSERT(queue->size == queue->capacity);
    TEST_ASSERT(queue->start == 0);
    TEST_ASSERT(queue->end == queue->capacity);
    // We should get "last" first, then everything from 0 to last-1,
    // then last+1 to capacity.
    TEST_CHECK(last == (intptr_t)queue->actors[0]);
    for (intptr_t i = 1; i < last; i++) {
        TEST_CHECK(i-1 == (intptr_t)queue->actors[i]);
    }
    for (intptr_t i = last+1; i < (intptr_t)queue->capacity; i++) {
        TEST_CHECK(i == (intptr_t)queue->actors[i]);
    }

    // First re-number everything from 0 up for ease.
    for (intptr_t i = 0; i < (intptr_t)queue->capacity; i++) {
        queue->actors[i] = (void*)i;
    }

    // Test growing the queue at capacity with end in the middle
    // (rotate start and end manually).
    intptr_t end = queue->end;
    intptr_t pivot = queue->capacity / 2;
    intptr_t pivot_value = (intptr_t)queue->actors[pivot];
    TEST_ASSERT(pivot == pivot_value);
    queue->start = (size_t)pivot;
    queue->end = (size_t)pivot;
    for (intptr_t i = n; i < n * 2; i++) {
        enqueue_actor(queue, (void*)i);
    }
    n = n * 2;
    TEST_ASSERT(queue->size == (size_t)n);
    TEST_ASSERT(queue->size == queue->capacity);
    TEST_ASSERT(queue->start == 0);
    TEST_ASSERT(queue->end == queue->capacity);
    // We should get pivot to end-1, then 0 to pivot-1,
    // then end to capacity.
    for (intptr_t i = 0; i < pivot; i++) {
        TEST_CHECK(i+pivot == (intptr_t)queue->actors[i]);
    }
    for (intptr_t i = pivot; i < end; i++) {
        TEST_CHECK(i-pivot == (intptr_t)queue->actors[i]);
    }
    for (intptr_t i = end; i < (intptr_t)queue->capacity; i++) {
        TEST_CHECK(i == (intptr_t)queue->actors[i]);
    }

    free_ActorQueue(queue);
}
