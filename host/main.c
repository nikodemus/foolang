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

#define FOO_ALLOC(type) \
  ((type*)foo_alloc(1, sizeof(type)))
#define FOO_ALLOC_ARRAY(n, type) \
  ((type*)foo_alloc((n), sizeof(type)))

#if 0
# define FOO_DEBUG(...) { printf(__VA_ARGS__); fflush(stdout); }
#else
# define FOO_DEBUG(...)
#endif

#define FOO_PANIC(...) { printf("PANIC" __VA_ARGS__); fflush(stdout); _Exit(1); }

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

void* foo_alloc(size_t n, size_t size) {
  void* new = calloc(n, size);
  if (new) {
    return new;
  } else {
    foo_abort("foo_alloc_failed");
  }
}

struct FooVtable;
struct FooBlock;

union FooDatum {
  struct Foo* object;
  struct FooBlock* block;
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

#include "generated_selectors.h"

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

struct FooContext {
  const char* info;
  struct FooContext* sender;
  struct Foo receiver;
  struct Foo* frame;
};

typedef struct Foo (*FooMethodFunction)(struct FooContext*, size_t, va_list);
typedef struct Foo (*FooBlockFunction)(struct FooContext*);

struct Foo foo_lexical_ref(struct FooContext* context, size_t index, size_t frame) {
  while (frame > 0) {
    assert(context->sender);
    context = context->sender;
    --frame;
  }
  return context->frame[index];
}
struct Foo* foo_frame_new(size_t size) {
  return FOO_ALLOC_ARRAY(size, struct Foo);
}

void foo_vargs_to_frame(size_t nargs, va_list vargs, struct Foo* frame) {
  for (size_t i = 0; i < nargs; ++i) {
    frame[i] = va_arg(vargs, struct Foo);
  }
}

struct FooMethod {
  struct FooSelector* selector;
  size_t argCount;
  size_t frameSize;
  FooMethodFunction function;
};

struct FooBlock {
  struct FooContext* context;
  size_t argCount;
  size_t frameSize;
  FooBlockFunction function;
};

struct Foo foo_Integer_new(int64_t n);

struct FooContext* foo_context_new_main(size_t frameSize) {
  struct FooContext* context = FOO_ALLOC(struct FooContext);
  context->info = "main";
  context->sender = NULL;
  context->receiver = foo_Integer_new(0); // should be: Main
  context->frame = foo_frame_new(frameSize);
  return context;
}

struct FooContext* foo_context_new_method(struct FooMethod* method, struct FooContext* sender, struct Foo receiver, size_t nargs) {
  if (method->argCount != nargs) {
    FOO_PANIC("Wrong number of arguments to %s. Wanted: %zu, got: %zu.",
              method->selector->name->data, method->argCount, nargs);
  }
  struct FooContext* context = FOO_ALLOC(struct FooContext);
  context->info = "method";
  context->sender = sender;
  context->receiver = receiver;
  context->frame = foo_frame_new(method->frameSize);
  return context;
}

struct FooContext* foo_context_new_block(struct FooBlock* block, size_t nargs) {
  if (block->argCount != nargs) {
    FOO_PANIC("Wrong number of arguments to block. Wanted: %zu, got: %zu.",
              block->argCount, nargs);
  }
  struct FooContext* context = FOO_ALLOC(struct FooContext);
  context->info = "block";
  context->sender = block->context;
  context->receiver = block->context->receiver;
  context->frame = foo_frame_new(block->frameSize);
  return context;
}

struct FooMethodArray {
  size_t size;
  struct FooMethod data[];
};

struct FooVtable {
  struct FooCString* name;
  struct FooMethodArray* methods;
};

struct Foo foo_vtable_typecheck(struct FooVtable* vtable, struct Foo obj) {
  if (vtable == obj.vtable) {
    return obj;
  } else {
    FOO_PANIC("Type error! Wanted: %s, got: %s\n",
              vtable->name->data, obj.vtable->name->data);
  }
}

struct FooMethod* foo_vtable_find_method(const struct FooVtable* vtable, const struct FooSelector* selector) {
  struct FooMethodArray* methods = vtable->methods;
  FOO_DEBUG("/foo_vtable_find_method(%s#%s)\n", vtable->name.data, selector->name->data);
  assert(methods);
  for (size_t i = 0; i < methods->size; ++i) {
    struct FooMethod* method = &methods->data[i];
    if (method->selector == selector) {
      return method;
    }
  }
  return NULL;
}

struct Foo foo_send(struct FooContext* sender,
                    const struct FooSelector* selector,
                    struct Foo receiver, size_t nargs, ...) {
  FOO_DEBUG("/foo_send(?, %s, ...)\n", selector->name->data);
  va_list arguments;
  va_start(arguments, nargs);
  assert(receiver.vtable);
  struct FooMethod* method = foo_vtable_find_method(receiver.vtable, selector);
  if (method) {
    struct FooContext* context = foo_context_new_method(method, sender, receiver, nargs);
    return method->function(context, nargs, arguments);
  } else {
    FOO_PANIC("foo_send: receiver does not understand message");
  }
}

struct FooVtable FOO_IntegerVtable;
struct FooVtable FOO_BlockVtable;

struct Foo foo_block_new(struct FooContext* context,
                         FooBlockFunction function,
                         size_t argCount,
                         size_t frameSize) {
  struct FooBlock* block = FOO_ALLOC(struct FooBlock);
  block->context = context;
  block->function = function;
  block->argCount = argCount;
  block->frameSize = frameSize;
  return (struct Foo){ .vtable = &FOO_BlockVtable, .datum = { .block = block } };
}

#include "generated_blocks.c"

struct Foo foo_Integer_new(int64_t n) {
  return (struct Foo){ .vtable = &FOO_IntegerVtable, .datum = { .int64 = n } };
}

struct Foo foo_Integer_method_debug(struct FooContext* ctx,
                                    __attribute__ ((unused)) size_t nargs,
                                    __attribute__ ((unused)) va_list args) {
  struct Foo receiver = ctx->receiver;
  printf("#<Integer %" PRId64 ">", receiver.datum.int64);
  return receiver;
}

struct Foo foo_Integer_method_add(struct FooContext* ctx,
                                  __attribute__ ((unused)) size_t nargs,
                                  va_list args) {
  struct Foo arg = foo_vtable_typecheck(&FOO_IntegerVtable, va_arg(args, struct Foo));
  return foo_Integer_new(ctx->receiver.datum.int64 + arg.datum.int64);
}

struct Foo foo_Integer_method_mul(struct FooContext* ctx,
                                  __attribute__ ((unused)) size_t nargs,
                                  va_list args) {
  struct Foo arg = foo_vtable_typecheck(&FOO_IntegerVtable, va_arg(args, struct Foo));
  return foo_Integer_new(ctx->receiver.datum.int64 * arg.datum.int64);
}

struct FooMethodArray FOO_IntegerBuiltinMethods =
  {
   .size = 3,
   .data = { (struct FooMethod){ .selector = &FOO_SELECTOR__debug,
                                 .argCount = 0,
                                 .frameSize = 0,
                                 .function = &foo_Integer_method_debug },
             (struct FooMethod){ .selector = &FOO_SELECTOR_add,
                                 .argCount = 1,
                                 .frameSize = 1,
                                 .function = &foo_Integer_method_add },
             (struct FooMethod){ .selector = &FOO_SELECTOR_mul,
                                 .argCount = 1,
                                 .frameSize = 1,
                                 .function = &foo_Integer_method_mul }}
  };

struct FooVtable FOO_IntegerVtable =
  {
   .name = &FOO_CSTRING("Integer"),
   .methods = &FOO_IntegerBuiltinMethods
  };

struct Foo foo_Block_method_value(struct FooContext* ctx, size_t nargs, va_list args) {
  struct FooBlock* block = ctx->receiver.datum.block;
  struct FooContext* context = foo_context_new_block(block, nargs);
  FOO_DEBUG("value 1\n");
  foo_vargs_to_frame(nargs, args, context->frame);
  FOO_DEBUG("value 2\n");
  return block->function(context);
}

struct FooMethodArray FOO_BlockBuiltinMethods =
  {
   .size = 1,
   .data = { (struct FooMethod){ .selector = &FOO_SELECTOR__value,
                                 .argCount = 0,
                                 .frameSize = 0,
                                 .function = &foo_Block_method_value } }
  };

struct FooVtable FOO_BlockVtable =
  {
   .name = &FOO_CSTRING("Block"),
   .methods = &FOO_BlockBuiltinMethods
  };

int main() {
  #include "generated_main.c"
  return 0;
}
