#ifndef __ACTOR_H_
#define __ACTOR_H_

#include "system.h"

enum ActorState {
  ActorReady = 0,
  ActorRunning = 1,
  ActorExiting = 2,
};

struct Actor {
  _Atomic enum ActorState state;
  char* bp;
  char* sp;
  char* stack;
  char* stacktop;
  void* data;
};

typedef char* (*ActorContinuation)(char*, struct Actor*);

struct Actor* make_Actor(ActorContinuation entry, void* data);
void free_Actor(struct Actor*);
enum ActorState run_actor_timeslice(struct Actor*);
void wait_for_actor_exit(struct Actor*, size_t);

#endif // __ACTOR_H_
