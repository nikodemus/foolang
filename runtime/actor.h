#ifndef __ACTOR_H_
#define __ACTOR_H_

struct Actor {
  size_t id;
};

struct Actor* make_Actor(size_t id);

#endif // __ACTOR_H_
