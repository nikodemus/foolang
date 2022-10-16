#include "actor.h"

#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>

#include "config.h"
#include "system.h"
#include "utils.h"

#undef NDEBUG
#include <assert.h>
#include <stdio.h>

void print_actor(struct Actor* actor) {
  printf("(struct Actor) { // addr = %p\n", actor);
  printf("    .state = %d,\n", actor->state);
  printf("    .bp =       %p,\n", actor->bp);
  printf("    .sp =       %p,\n", actor->sp);
  printf("    .stack =    %p,\n", actor->stack);
  printf("    .stacktop = %p,\n", actor->stacktop);
  printf("    .data = %p\n", actor->data);
  printf("}\n");
}

struct Actor* make_Actor(ActorContinuation entry, void* data) {
  struct Actor* actor = malloc(sizeof(struct Actor));
  if (!actor)
    fatal("Could not allocate memory for an actor.");
  atomic_init(&actor->state, ActorReady);
  actor->stack = malloc(FOO_ACTOR_STACK_BYTES);
  if (!actor->stack)
    fatal("Could not allocate memory for an actor's stack.");
  actor->bp = 0;
  actor->sp = actor->stack;
  actor->stacktop = actor->stack + FOO_ACTOR_STACK_BYTES;
  // Set the entry point on stack, and save initial data.
  *(ActorContinuation*)(actor->sp) = entry;
  actor->data = data;
  return actor;
}

void free_Actor(struct Actor* actor) {
  free(actor->stack);
  free(actor);
}

enum ActorState run_actor_timeslice(struct Actor* actor) {
  char* sp = actor->sp;
  actor->sp = NULL;
  atomic_store(&actor->state, ActorRunning);

  // To yield the continuation can return a zero.
  for (size_t i = 0; i <= FOO_ACTOR_TIMESLICE; i++) {
    sp = (*(ActorContinuation*)(sp))(sp, actor);
    if (!sp)
      break;
  }

  if (sp) {
    actor->sp = sp;
  } else if (actor->state == ActorRunning) {
    assert(actor->sp); // Yielding actors must save sp!
    atomic_store(&actor->state, ActorReady);
  }

  return actor->state;
}

void wait_for_actor_exit(struct Actor* actor, size_t timeout_ms) {
  if (timeout_ms) {
    while (timeout_ms--) {
      if (atomic_load(&actor->state) == ActorExiting) {
        return;
      }
      system_sleep_ms(1);
    }
  } else {
    for (;;) {
      if (atomic_load(&actor->state) == ActorExiting) {
        return;
      }
      system_sleep_ms(1);
    }
  }
}
