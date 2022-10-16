#ifndef __FOOLANG_H_
#define __FOOLANG_H_

#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>

#include "actor.h"
#include "datum.h"
#include "error.h"
#include "executor_pool.h"

struct FooBytes {
  size_t size;
  char data[];
};

struct FooSelector {
  struct FooBytes* name;
};

struct FooMethod {
  struct FooClass* home;
  struct FooSelector* selector;
  ActorContinuation method_function;
};

struct FooMethodDictionary {
  size_t size;
  struct FooMethod* data[];
};

struct FooClass {
  struct FooBytes* name;
  struct FooClass* own_class;
  struct FooMethodDictionary* methods;
};

#define OBJS(n) \
  (n)

#define IMMS(n) \
  (((int64_t)n) << 32)

#endif // __FOOLANG_H_
#ifndef __FOO_H_
#define __FOO_H_

#include "actor.h"
#include "actor_queue.h"
#include "executor.h"
#include "executor_pool.h"
#include "system.h"

#endif // __FOO_H_
