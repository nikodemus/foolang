#include <inttypes.h>
#include <setjmp.h>
#include <stdio.h>
#include <stdlib.h>

#undef NDEBUG
#include <assert.h>

#include "pi_config.h"

struct Actor {
  char* sp;
  char* stack;
  char* stacktop;
  size_t stacksize;
  size_t timeslice;
  jmp_buf* yieldpoint;
};

typedef char* (*Continuation)(char*, struct Actor*);

#define PUSH_POINTER(sp, pointer) \
  { sp += sizeof(void*);          \
    *((void**)sp) = pointer; }    \

#define POP_POINTER(sp)           \
  { sp -= sizeof(void*); }        \

#define PUSH_I64(sp, i)             \
  { sp += sizeof(int64_t);          \
    *((int64_t*)sp) = (int64_t)i; } \

#define POP_I64(sp)                              \
  ({ int64_t tmpval = *((int64_t*)sp);           \
     sp -= sizeof(int64_t);                      \
     tmpval; })                                  \

// Assumes just immediates!
#define ARG_I64(sp, n)                            \
  ((int64_t*)sp)[-(n+1)]                          \

#define ARG_F64(sp, n)                           \
  ((double*)sp)[-(n+1)]                          \

#define YIELD(sp, actor)                         \
  ({ actor->sp = sp;                             \
     longjmp(*actor->yieldpoint, 1);             \
     NULL; })                                    \

#define ARG_OBJS(n) (((size_t)n) << 32)
#define ARG_PTRS(n) (((size_t)n) << 16)
#define ARG_IMMS(n) ((size_t)n)

void run_actor(struct Actor* actor) {
  char* sp = actor->sp;
  jmp_buf yieldpoint;

  actor->sp = NULL;
  actor->yieldpoint = &yieldpoint;

  if (setjmp(yieldpoint)) {
    // printf("XXX: timeslice yielded\n");
    goto exit;
  }

  // printf("XXX: timeslice start\n");
  size_t timeslice = actor->timeslice;
  for (size_t i = 0; i <= timeslice; i++) {
    sp = (*(Continuation*)(sp))(sp, actor);
  }
  actor->sp = sp;
  // printf("XXX: timeslice end\n");

 exit:
  actor->yieldpoint = NULL;
}

const size_t DEFAULT_STACKSIZE = 1024;
const size_t DEFAULT_TIMESLICE = 1024;

void panic(char* what) {
  printf("PANIC: %s\n", what);
  exit(1);
}

struct Actor* make_actor(Continuation entry) {
  struct Actor* new = malloc(sizeof(struct Actor));
  if (!new)
    panic("Could not allocate memory for an actor");
  new->stacksize = DEFAULT_STACKSIZE;
  new->stack = malloc(new->stacksize);
  if (!new->stack)
    panic("Could not allocate memory for an actor's stack");
  new->sp = new->stack;
  new->stacktop = new->stack + new->stacksize;
  new->timeslice = DEFAULT_TIMESLICE;
  *(Continuation*)(new->sp) = entry;
  return new;
}

double random_double() {
  return (double)rand()/(double)RAND_MAX;
}


char* exit_continuation(char* sp, struct Actor* actor) {
  (void)sp;
  (void)actor;
  exit(0);
  return NULL;
}


char* pi_loop_body(char* sp, struct Actor* actor) {
  (void)actor;
  POP_POINTER(sp);
  POP_I64(sp);
  double x = random_double();
  double y = random_double();
  x = x*x;
  y = y*y;
  if (x + y < 1.0) {
    ARG_I64(sp, 3)++;
  }
  return sp;
}


char* pi_loop_exit(char* sp, struct Actor* actor) {
  (void)actor;
  POP_POINTER(sp);
  POP_I64(sp);
  int64_t inside = POP_I64(sp);
  double ratio = 4.0 * (double)inside / N_ITERATIONS;
  ARG_F64(sp, 1) = ratio;
  return sp;
}


char* pi_loop_test(char* sp, struct Actor* actor) {
  (void)actor;
  if (ARG_I64(sp, 2) < ARG_I64(sp, 1)) {
    ARG_I64(sp, 2)++;
    PUSH_I64(sp, ARG_IMMS(0));
    PUSH_POINTER(sp, pi_loop_body);
  } else {
    POP_POINTER(sp); // this
    POP_I64(sp);     // arginfo
    POP_I64(sp);     // n
    POP_I64(sp);     // i
    PUSH_I64(sp, ARG_IMMS(1));
    PUSH_POINTER(sp, pi_loop_exit);
  }
  return sp;
}


char* pi_entry(char* sp, struct Actor* actor) {
  (void)actor;
  POP_POINTER(sp);
  POP_I64(sp);
  PUSH_I64(sp, 0);       // inside = 0
  PUSH_I64(sp, 1);       // i = 1
  PUSH_I64(sp, N_ITERATIONS);
  PUSH_I64(sp, ARG_IMMS(2));
  PUSH_POINTER(sp, pi_loop_test);
  return sp;
}


char* main_exit(char* sp, struct Actor* actor) {
  (void)actor;
  double f = ARG_F64(sp, 1);
  printf("    pi = %.2f\n", f);
  POP_POINTER(sp);
  POP_I64(sp);
  POP_I64(sp);
  PUSH_I64(sp, ARG_IMMS(0));
  PUSH_POINTER(sp, exit_continuation);
  return sp;
}


char* main_entry(char* sp, struct Actor* actor) {
  (void)actor;
  PUSH_I64(sp, 0); // retval
  PUSH_I64(sp, ARG_IMMS(1));
  PUSH_POINTER(sp, main_exit);
  PUSH_I64(sp, ARG_IMMS(0));
  PUSH_POINTER(sp, pi_entry);
  return sp;
}


int main() {
  struct Actor* main = make_actor(main_entry);
  for (;;) {
    run_actor(main);
  }
  return 0;
}
