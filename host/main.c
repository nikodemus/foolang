#include <stdint.h>
#include <inttypes.h>
#include <string.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <setjmp.h>
#include <errno.h>
#include <stdarg.h>
#undef NDEBUG
#include <assert.h>

#define FOO_ALLOC(type) \
  ((type*)foo_alloc(1, sizeof(type)))
#define FOO_ALLOC_ARRAY(n, type) \
  ((type*)foo_alloc((n), sizeof(type)))

#if 0
# define FOO_DEBUG(...) { fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); fflush(stderr); }
#else
# define FOO_DEBUG(...)
#endif

#define FOO_PANIC(...) { printf("PANIC: " __VA_ARGS__); fflush(stdout); _Exit(1); }

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
    foo_abort("foo_alloc failed!");
  }
}

struct FooVtable;
struct FooBlock;
struct FooClass;

union FooDatum {
  struct Foo* object;
  struct FooBlock* block;
  struct FooClass* class;
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

struct FooSlot {
  const struct FooCString* name;
};

struct FooLayout {
  size_t size;
  struct FooSlot slots[];
};

// FIXME: Terribly messy
struct FooContext {
  const char* info;
  struct FooContext* sender;
  struct Foo receiver;
  size_t size;
  struct Foo* frame;
  struct FooContext* return_context;
  jmp_buf* ret;
  struct Foo ret_value;
};

char* foo_debug_context(struct FooContext* ctx) {
  const int size = 1024;
  char* s = (char*)malloc(size+1);
  assert(s);
  snprintf(s, size, "{ .info = %s, .size = %zu, .frame = %p, .ret = %p }",
           ctx->info, ctx->size, ctx->frame, ctx->ret);
  return s;
}

typedef struct Foo (*FooMethodFunction)(struct FooContext*, size_t, va_list);
typedef struct Foo (*FooBlockFunction)(struct FooContext*);

struct Foo foo_lexical_ref(struct FooContext* context, size_t index, size_t frame) {
  FOO_DEBUG("/lexical_ref(index=%zu, frame=%zu)", index, frame);
  while (frame > 0) {
    assert(context->sender);
    context = context->sender;
    --frame;
  }
  struct Foo res = context->frame[index];
  assert(res.vtable);
  return res;
}
struct Foo* foo_frame_new(size_t size) {
  return FOO_ALLOC_ARRAY(size, struct Foo);
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
  context->size = frameSize;
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
  context->size = method->frameSize;
  context->frame = foo_frame_new(method->frameSize);
  context->return_context = context;
  return context;
}

void foo_context_method_unwind(struct FooContext** ctx) {
  (*ctx)->ret = NULL;
}

struct FooContext* foo_context_new_block(struct FooContext* ctx) {
  struct FooContext* context = FOO_ALLOC(struct FooContext);
  struct FooBlock* block = ctx->receiver.datum.block;
  context->info = "block";
  context->sender = block->context;
  context->receiver = block->context->receiver;
  context->size = block->frameSize;
  context->frame = foo_frame_new(block->frameSize);
  context->return_context = context->sender;
  for (size_t i = 0; i < block->argCount; ++i)
    context->frame[i] = ctx->frame[i];
  return context;
}

struct FooMethodArray {
  size_t size;
  struct FooMethod data[];
};

struct FooVtable {
  struct FooCString* name;
  struct FooMethodArray* methods;
  struct Foo* classptr;
};

struct FooClass {
  // struct FooLayout* instanceSlots;
  struct FooVtable* instanceVtable;
};

struct Foo foo_vtable_typecheck(struct FooVtable* vtable, struct Foo obj) {
  if (vtable == obj.vtable)
    return obj;
  assert(vtable);
  assert(obj.vtable);
  FOO_PANIC("Type error! Wanted: %s, got: %s",
            vtable->name->data, obj.vtable->name->data);
}

struct FooMethod* foo_vtable_find_method(const struct FooVtable* vtable, const struct FooSelector* selector) {
  assert(vtable);
  struct FooMethodArray* methods = vtable->methods;
  // FOO_DEBUG("/foo_vtable_find_method(%s#%s)", vtable->name->data, selector->name->data);
  assert(methods);
  for (size_t i = 0; i < methods->size; ++i) {
    struct FooMethod* method = &methods->data[i];
    if (method->selector == selector) {
      return method;
    }
  }
  return NULL;
}

struct Foo foo_return(struct FooContext* ctx, struct Foo value) __attribute__ ((noreturn));
struct Foo foo_return(struct FooContext* ctx, struct Foo value) {
  FOO_DEBUG("/foo_return(...)");
  if (ctx->return_context->ret) {
    ctx->return_context->ret_value = value;
    longjmp(*ctx->return_context->ret, 1);
    FOO_PANIC("longjmp() fell through!")
  } else {
    FOO_PANIC("Cannot return from here: %s", foo_debug_context(ctx));
  }
}

struct Foo foo_send(struct FooContext* sender,
                    const struct FooSelector* selector,
                    struct Foo receiver, size_t nargs, ...) {
  FOO_DEBUG("/foo_send(?, %s, ...)", selector->name->data);
  va_list arguments;
  va_start(arguments, nargs);
  assert(receiver.vtable);
  struct FooMethod* method = foo_vtable_find_method(receiver.vtable, selector);
  if (method) {
    struct FooContext* context __attribute__((cleanup(foo_context_method_unwind)));
    context = foo_context_new_method(method, sender, receiver, nargs);
    jmp_buf ret;
    context->ret = &ret;
    int jmp = setjmp(ret);
    if (jmp) {
      FOO_DEBUG("/foo_send -> non-local return from %s", selector->name->data);
      return context->ret_value;
    } else {
      struct Foo res = method->function(context, nargs, arguments);
      FOO_DEBUG("/foo_send -> local return from %s", selector->name->data);
      return res;
    }
  } else {
    FOO_PANIC("%s does not understand: #%s",
              receiver.vtable->name->data, selector->name->data);
  }
}

// FIXME: These forward declarations should be generated
struct FooVtable FooInstanceVtable_Integer;
struct FooVtable FooInstanceVtable_Block;

struct Foo foo_block_new(struct FooContext* context,
                         FooBlockFunction function,
                         size_t argCount,
                         size_t frameSize) {
  struct FooBlock* block = FOO_ALLOC(struct FooBlock);
  block->context = context;
  block->function = function;
  block->argCount = argCount;
  block->frameSize = frameSize;
  return (struct Foo){ .vtable = &FooInstanceVtable_Block, .datum = { .block = block } };
}

struct Foo foo_Integer_new(int64_t n) {
  return (struct Foo){ .vtable = &FooInstanceVtable_Integer, .datum = { .int64 = n } };
}

#include "generated_blocks.c"

#include "generated_classes.c"

#include "generated_main.c"
