#include <stdint.h>
#include <inttypes.h>
#include <string.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <errno.h>
#include <stdarg.h>
#undef NDEBUG
#include <assert.h>

#define FOO_ALLOC(type) ((type*)foo_alloc(sizeof(type)))

void foo_panic(const char* message) __attribute__ ((noreturn));
void foo_panic(const char* message) {
  printf("PANIC: %s", message);
  fflush(stdout);
  fflush(stderr);
  _Exit(1);
}

void foo_unimplemented(const char* message) __attribute__ ((noreturn));
void foo_unimplemented(const char* message) {
  printf("UNIMPLEMENTED: %s", message);
  fflush(stdout);
  fflush(stderr);
  _Exit(1);
}

void foo_abort(const char* message) __attribute__ ((noreturn));
void foo_abort(const char* message) {
  perror(message);
  fflush(stdout);
  fflush(stderr);
  _Exit(1);
}

void* foo_alloc(size_t size) {
  void* new = calloc(1, size);
  if (new) {
    return new;
  } else {
    foo_abort("foo_alloc_failed");
  }
}

void foo_debug(char* message) {
  printf("DEBUG: %s\n", message);
  fflush(stdout);
  fflush(stderr);
}

struct FooVtable;

union FooDatum {
  struct Foo* object;
  int64_t int64;
};

struct Foo {
  struct FooVtable* vtable;
  union FooDatum datum;
};

/** Out of line allocation of data for compatibility with string literals.
 */
struct FooCString {
  size_t size;
  char* data;
};

#define FOO_CSTRING(literal) \
  ((struct FooCString){ .size = sizeof(literal)-1, .data = literal })

bool foo_cstring_equal(const struct FooCString* a, const struct FooCString* b) {
  return a->size == b->size && !memcmp(a->data, b->data, a->size);
}

/** Simple intrusive list for interning. O(N), but fine to start with.
 */
struct FooSelector {
  const struct FooCString* name;
  struct FooSelector* next;
};

struct FooSelector FOO_SELECTOR_debug = { .name = &FOO_CSTRING("debug"), .next = NULL };
#include "selectors.h"

struct FooSelector* foo_intern_new_selector(const struct FooCString* name) {
  struct FooSelector* new = FOO_ALLOC(struct FooSelector);
  new->name = name;
  new->next = FOO_InternedSelectors;
  FOO_InternedSelectors = new;
  return new;
}

struct FooSelector* foo_intern(const struct FooCString* name) {
  struct FooSelector* selector = FOO_InternedSelectors;
  while (selector != NULL) {
    if (foo_cstring_equal(selector->name, name)) {
      return selector;
    } else {
      selector = selector->next;
    }
  }
  return foo_intern_new_selector(name);
}

struct FooArray {
  size_t size;
  struct Foo data[];
};

typedef struct Foo (*FooMethodFunction)(struct Foo, va_list);

struct FooMethod {
  struct FooSelector* selector;
  FooMethodFunction function;
};

struct FooMethodArray {
  size_t size;
  struct FooMethod data[];
};

struct FooVtable {
  struct FooCString name;
  struct FooMethodArray* methods;
};

struct FooMethod* foo_vtable_find_method(const struct FooVtable* vtable, const struct FooSelector* selector) {
  struct FooMethodArray* methods = vtable->methods;
  assert(methods);
  for (size_t i = 0; i < methods->size; ++i) {
    struct FooMethod* method = &methods->data[i];
    if (method->selector == selector) {
      return method;
    }
  }
  return NULL;
}

struct FooVtable FOO_IntegerVtable;

/**
   TODO: pass in context
 */

struct Foo foo_Integer_method_debug(struct Foo receiver,
                                    __attribute__ ((unused)) va_list arguments) {
  printf("#<Integer %" PRId64 ">", receiver.datum.int64);
  return receiver;
}

struct Foo foo_Integer_method__add_(struct Foo receiver,
                                    __attribute__ ((unused)) va_list arguments) {
  struct Foo arg = va_arg(arguments, struct Foo);
  return (struct Foo){ .vtable = &FOO_IntegerVtable, .datum = { .int64 = receiver.datum.int64 + arg.datum.int64 } };
}

struct Foo foo_Integer_method__mul_(struct Foo receiver,
                                    __attribute__ ((unused)) va_list arguments) {
  struct Foo arg = va_arg(arguments, struct Foo);
  return (struct Foo){ .vtable = &FOO_IntegerVtable, .datum = { .int64 = receiver.datum.int64 * arg.datum.int64 } };
}

struct FooMethodArray FOO_IntegerBuiltinMethods =
  {
   .size = 3,
   .data = { (struct FooMethod){ .selector = &FOO_SELECTOR_debug,
                                 .function = &foo_Integer_method_debug },
             (struct FooMethod){ .selector = &FOO_SELECTOR__add_,
                                 .function = &foo_Integer_method__add_ },
             (struct FooMethod){ .selector = &FOO_SELECTOR__mul_,
                                 .function = &foo_Integer_method__mul_ }}
  };

struct FooVtable FOO_IntegerVtable =
  {
   .name = FOO_CSTRING("Integer"),
   .methods = &FOO_IntegerBuiltinMethods
  };

struct Foo foo_send(const struct FooSelector* selector, struct Foo receiver, ...) {
  va_list arguments;
  va_start(arguments, receiver);
  struct FooMethod* method = foo_vtable_find_method(receiver.vtable, selector);
  if (method) {
    return method->function(receiver, arguments);
  } else {
    printf("MSg: %s\n", selector->name->data);
    foo_panic("foo_send: receiver does not understand message!");
  }
}

int main() {
  #include "main.h"
  return 0;
}
