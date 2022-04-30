#include "actor_queue.h"

#undef NDEBUG
#include <assert.h>
#include <stdlib.h>
#include <stdio.h>

#include "config.h"
#include "system.h"
#include "utils.h"

size_t queue_size(struct ActorQueue* queue) {
  return queue->size;
}

struct ActorQueue* make_ActorQueue() {
  size_t size = FOO_INITIAL_ACTOR_QUEUE_SIZE;
  struct ActorQueue* queue = malloc(sizeof(struct ActorQueue));
  if (!queue)
    fatal("Could not allocate memory for actor queue.");
  queue->actors = malloc(size * sizeof(struct Actor*));
  if (!queue->actors)
    fatal("Could not allocate memory for actor queue actors.");
  queue->lock = make_SystemLock();
  queue->start = 0;
  queue->end = 0;
  queue->size = 0;
  queue->capacity = size;
  return queue;
}

void enqueue_actor(struct ActorQueue* queue, struct Actor* actor) {
  system_lock(queue->lock);

  // Case 1: empty
  if (queue->size == 0)
    goto enqueue;

  // Case 2: full
  if (queue->size == queue->capacity)
    goto grow;

  // Case 3: start <= end, no wraparound yet.
  //
  // Either there is start at the end (end < capacity),
  // or there is space in front after wraparound (start > 0),
  //
  if (queue->start <= queue->end) {
    if (queue->end == queue->capacity) {
      assert(queue->start > 0);
      queue->end = 0;
    }
    goto enqueue;
  }

  // Case 3: wraparound.
  goto enqueue;

 grow:
  // Allocate new memory.
  size_t new_capacity = minz(queue->capacity * 2, queue->capacity + 1024);
  struct Actor** new_actors = malloc( sizeof(struct Actor*) * new_capacity);
  if (!new_actors)
    fatal("Could not allocate space for new actors when growing queue.");
  // Copy existing actors to beginning of the new memory.
  //
  // Either [start...end] or [...end, start....]
  //
  // In either case we copy from start to capacity to the beginning of the new
  // memory, and from zero to start to the end of the new memory.
  size_t i = 0;
  size_t j = queue->start;
  while (j < queue->capacity)
    new_actors[i++] = queue->actors[j++];
  size_t k = 0;
  while (k < queue->start)
    new_actors[i++] = queue->actors[k++];
  assert(i == queue->capacity);
  // Free old memory.
  free(queue->actors);
  // Put everything in place.
  queue->capacity = new_capacity;
  queue->start = 0;
  queue->end = i;
  queue->actors = new_actors;
  // Fallthrough to enqueue!

 enqueue:
    queue->actors[queue->end++] = actor;
    queue->size++;
    assert(queue->end <= queue->capacity);
    assert(queue->size <= queue->capacity);
    system_unlock(queue->lock);
}

struct Actor* dequeue_actor(struct ActorQueue* queue) {
  struct Actor* actor = NULL;
  system_lock(queue->lock);
  // Case 1: empty queue
  if (queue->size == 0)
    goto done;
  // Case 2: pop first
  actor = queue->actors[queue->start++];
  queue->size--;

 done:
  system_unlock(queue->lock);
  return actor;
}

