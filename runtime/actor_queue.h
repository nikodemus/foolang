#ifndef __ACTOR_QUEUE_H_
#define __ACTOR_QUEUE_H_

#include "actor.h"
#include "system.h"

struct ActorQueue {
  SystemLock_t lock;
  // Inclusive start, exclusive end. Both can wrap around:
  // start == end means either empty of full -- must consult size.
  size_t start;
  size_t end;
  size_t size;
  size_t capacity;
  struct Actor** actors;
};

struct ActorQueue* make_ActorQueue();
void enqueue_actor(struct ActorQueue* queue, struct Actor* actor);
struct Actor* dequeue_actor(struct ActorQueue* queue);
size_t queue_size(struct ActorQueue* queue);

#endif // __ACTOR_QUEUE_H_
