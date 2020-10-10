#include <float.h>
#include <stdint.h>
#include <inttypes.h>
#include <string.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <setjmp.h>
#include <errno.h>
#include <stdarg.h>
#include <stdbool.h>
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
struct FooBytes;

// FIXME: fold all pointers into a single void*
union FooDatum {
  struct Foo* object;
  struct FooBlock* block;
  struct FooClass* class;
  struct FooBytes* bytes;
  int64_t int64;
  double float64;
  bool boolean;
};

struct Foo {
  struct FooVtable* vtable;
  union FooDatum datum;
};

/** Out of line allocation of data for compatibility with C string literals.
 *
 *  FIXME: I'd really like to replace these with String.
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

struct FooBytes {
  size_t size;
  uint8_t data[];
};

struct FooSlot {
  const struct FooCString* name;
};

struct FooLayout {
  size_t size;
  struct FooSlot slots[];
};

struct FooProcess {
  size_t size;
  struct Foo vars[];
};

struct FooProcess* foo_process_new(size_t size) {
  struct FooProcess* process
    = foo_alloc(1, sizeof(struct FooProcess) + size * sizeof(struct Foo));
  process->size = size;
  return process;
}

struct FooContext;
struct FooCleanup;
typedef void (*FooCleanupFunction)(struct FooContext*, struct FooCleanup*);

struct FooCleanup {
  FooCleanupFunction function;
  struct FooCleanup* next;
};

struct FooFinally {
  struct FooCleanup cleanup;
  struct FooBlock* block;
};

struct FooUnbind {
  struct FooCleanup cleanup;
  size_t index;
  struct Foo value;
};

struct FooContext {
  const char* info;
  struct Foo receiver;
  struct FooContext* sender;
  struct FooContext* outer_context;
  // FIXME: Doesn't really belong in context, but easier right now.
  struct FooProcess* process;
  struct FooCleanup* cleanup;
  // Only for methods, for others this is NULL.
  jmp_buf* ret;
  size_t size;
  struct Foo frame[];
};

struct FooContext* foo_alloc_context(size_t size) {
  struct FooContext* ctx
    = foo_alloc(1, sizeof(struct FooContext) + size * sizeof(struct Foo));
  ctx->size = size;
  return ctx;
}

char* foo_debug_context(struct FooContext* ctx) {
  const int size = 1024;
  char* s = (char*)malloc(size+1);
  assert(s);
  snprintf(s, size, "{ .info = %s, .size = %zu, .outer_context = %p }",
           ctx->info, ctx->size, ctx->outer_context);
  return s;
}

typedef struct Foo (*FooMethodFunction)(struct FooContext*, size_t, va_list);
typedef struct Foo (*FooBlockFunction)(struct FooContext*);

struct Foo foo_lexical_ref(struct FooContext* context, size_t index, size_t frame) {
  FOO_DEBUG("/lexical_ref(index=%zu, frame=%zu)", index, frame);
  while (frame > 0) {
    assert(context->outer_context);
    context = context->outer_context;
    --frame;
  }
  assert(index < context->size);
  struct Foo res = context->frame[index];
  assert(res.vtable);
  return res;
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

// Forward declarations for vtables are in generated_classes, but we're going
// to define a few builtin ctors first that need some of them.
struct FooVtable FooInstanceVtable_Block;
struct FooVtable FooInstanceVtable_Boolean;
struct FooVtable FooInstanceVtable_Float;
struct FooVtable FooInstanceVtable_Integer;
struct FooVtable FooInstanceVtable_String;
struct Foo foo_Float_new(double f);
struct Foo foo_Integer_new(int64_t n);
struct Foo foo_String_new(size_t len, const char* s);
struct Foo foo_vtable_typecheck(struct FooVtable* vtable, struct Foo obj);
struct FooContext* foo_context_new_block(struct FooContext* ctx);

struct FooContext* foo_context_new_main(struct FooProcess* process) {
  struct FooContext* context = foo_alloc_context(0);
  context->info = "main";
  context->sender = NULL;
  context->receiver = foo_Integer_new(0); // should be Main?
  context->size = 0;
  context->outer_context = NULL;
  context->process = process;
  return context;
}

struct FooContext* foo_context_new_method(const struct FooMethod* method, struct FooContext* sender, struct Foo receiver, size_t nargs) {
  if (method->argCount != nargs) {
    FOO_PANIC("Wrong number of arguments to %s. Wanted: %zu, got: %zu.",
              method->selector->name->data, method->argCount, nargs);
  }
  struct FooContext* context = foo_alloc_context(method->frameSize);
  context->info = "method";
  context->sender = sender;
  context->receiver = receiver;
  context->outer_context = NULL;
  context->process = sender->process;
  return context;
}

struct FooContext* foo_context_new_block(struct FooContext* sender) {
  struct FooBlock* block = sender->receiver.datum.block;
  struct FooContext* context = foo_alloc_context(block->frameSize);
  context->info = "block";
  context->sender = sender;
  context->receiver = block->context->receiver;
  context->outer_context = block->context;
  context->process = sender->process;
  for (size_t i = 0; i < block->argCount; ++i)
    context->frame[i] = sender->frame[i];
  return context;
}

struct FooContext* foo_context_new_unwind(struct FooContext* ctx, struct FooBlock* block) {
  struct FooContext* context = foo_alloc_context(block->frameSize);
  context->info = "#finally:";
  context->sender = ctx;
  context->receiver = block->context->receiver;
  context->outer_context = block->context;
  assert(block->argCount == 0);
  return context;
}

void foo_cleanup(struct FooContext* sender) {
  while (sender->cleanup) {
    struct FooCleanup* cleanup = sender->cleanup;
    sender->cleanup = cleanup->next;
    cleanup->function(sender, cleanup);
  }
}

void foo_finally(struct FooContext* sender, struct FooCleanup* cleanup) {
  struct FooBlock* block = ((struct FooFinally*)cleanup)->block;
  // FIXME: Could stack-allocate this context.
  struct FooContext* block_ctx = foo_context_new_unwind(sender, block);
  block->function(block_ctx);
}

void foo_unbind(struct FooContext* sender, struct FooCleanup* cleanup) {
  struct FooUnbind* unbind = (struct FooUnbind*)cleanup;
  sender->process->vars[unbind->index] = unbind->value;
}

struct FooVtable {
  struct FooCString* name;
  struct Foo* classptr;
  size_t size;
  struct FooMethod methods[];
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

const struct FooMethod* foo_vtable_find_method(const struct FooVtable* vtable, const struct FooSelector* selector) {
  assert(vtable);
  // FOO_DEBUG("/foo_vtable_find_method(%s#%s)", vtable->name->data, selector->name->data);
  for (size_t i = 0; i < vtable->size; ++i) {
    const struct FooMethod* method = &vtable->methods[i];
    if (method->selector == selector) {
      return method;
    }
  }
  return NULL;
}

struct Foo foo_return(struct FooContext* ctx, struct Foo value) __attribute__ ((noreturn));
struct Foo foo_return(struct FooContext* ctx, struct Foo value) {
  FOO_DEBUG("/foo_return(%s...)", foo_debug_context(ctx));
  struct FooContext* return_context = ctx;
  while (return_context->outer_context) {
    return_context = return_context->outer_context;
  }
  while (ctx != return_context) {
    foo_cleanup(ctx);
    ctx = ctx->sender;
  }
  foo_cleanup(ctx);
  return_context->receiver = value;
  longjmp(*(jmp_buf*)return_context->ret, 1);
  FOO_PANIC("longjmp() fell through!")
}

struct Foo foo_send(struct FooContext* sender,
                    const struct FooSelector* selector,
                    struct Foo receiver, size_t nargs, ...) {
  FOO_DEBUG("/foo_send(?, %s, ...)", selector->name->data);
  va_list arguments;
  va_start(arguments, nargs);
  assert(receiver.vtable);
  const struct FooMethod* method = foo_vtable_find_method(receiver.vtable, selector);
  if (method) {
    struct FooContext* context = foo_context_new_method(method, sender, receiver, nargs);
    jmp_buf ret;
    context->ret = &ret;
    int jmp = setjmp(ret);
    if (jmp) {
      FOO_DEBUG("/foo_send -> non-local return from %s", selector->name->data);
      struct Foo return_value = context->receiver;
      context->receiver = receiver;
      return return_value;
    } else {
      struct Foo res = method->function((struct FooContext*)context, nargs, arguments);
      FOO_DEBUG("/foo_send -> local return from %s", selector->name->data);
      return res;
    }
  } else {
    FOO_PANIC("%s does not understand: #%s",
              receiver.vtable->name->data, selector->name->data);
  }
}

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

struct Foo FooGlobal_True =
  {
   .vtable = &FooInstanceVtable_Boolean,
   .datum = { .boolean = 1 }
  };

struct Foo FooGlobal_False =
  {
   .vtable = &FooInstanceVtable_Boolean,
   .datum = { .boolean = 0 }
  };

struct Foo foo_Boolean_new(bool t) {
  return (struct Foo){ .vtable = &FooInstanceVtable_Boolean, .datum = { .boolean = t } };
}

struct Foo foo_Integer_new(int64_t n) {
  return (struct Foo){ .vtable = &FooInstanceVtable_Integer, .datum = { .int64 = n } };
}

struct Foo foo_Float_new(double f) {
  return (struct Foo){ .vtable = &FooInstanceVtable_Float, .datum = { .float64 = f } };
}

struct Foo foo_String_new(size_t len, const char* s) {
  struct FooBytes* bytes = (struct FooBytes*)foo_alloc(1, sizeof(struct FooBytes) + len + 1);
  bytes->size = len;
  memcpy(bytes->data, s, len);
  return (struct Foo) { .vtable = &FooInstanceVtable_String, .datum = { .bytes = bytes } };
}

#include "generated_constants.c"
#include "generated_blocks.c"
#include "generated_classes.c"
#include "generated_main.c"
