#include "actor.h"

#include <stdlib.h>

#include "utils.h"

struct Actor* make_Actor(size_t id) {
  struct Actor* actor = malloc(sizeof(struct Actor*));
  if (!actor)
    fatal("Could not allocate memory for an actor.");
  actor->id = id;
  return actor;
}
