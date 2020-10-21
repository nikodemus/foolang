#include <float.h>
#include <stdint.h>
#include <inttypes.h>
#include <string.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdlib.h>
#include <stdio.h>
#include <setjmp.h>
#include <errno.h>
#include <stdarg.h>
#include <stdbool.h>
#undef NDEBUG
#include <assert.h>

#ifdef _WIN32
#include <io.h>
#include <fcntl.h>
#endif

#define PTR(type, datum) \
  ((struct type*)datum.ptr)

#if 0
# define FOO_DEBUG(...) { fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); fflush(stderr); }
#else
# define FOO_DEBUG(...)
#endif

#define FOO_PANIC(...) { printf("PANIC: " __VA_ARGS__); fflush(stdout); _Exit(1); }

struct FooContext;
void* foo_alloc(size_t bytes);
void foo_maybe_gc(struct FooContext* ctx);

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

union FooDatum {
  void* ptr;
  int64_t int64;
  double float64;
  int64_t boolean; // Make sure there's no junk in high bits
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
  struct FooSelector* new = calloc(1, sizeof(struct FooSelector));
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
  struct FooArray* vars;
  struct FooCleanup* cleanup;
  struct Foo return_value;
  // Only for methods, for others this is NULL.
  jmp_buf* ret;
  size_t size;
  struct Foo frame[];
};

struct FooContext* foo_alloc_context(size_t size) {
  struct FooContext* ctx
    = foo_alloc(sizeof(struct FooContext) + size * sizeof(struct Foo));
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

typedef struct Foo (*FooMethodFunction)(struct FooContext*);
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

struct Foo foo_lexical_set(struct FooContext* context, size_t index, size_t frame, struct Foo value) {
  FOO_DEBUG("/lexical_set(index=%zu, frame=%zu, ...)", index, frame);
  while (frame > 0) {
    assert(context->outer_context);
    context = context->outer_context;
    --frame;
  }
  assert(index < context->size);
  context->frame[index] = value;
  return value;
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
struct FooVtable FooInstanceVtable_Array;
struct FooVtable FooInstanceVtable_Block;
struct FooVtable FooInstanceVtable_Boolean;
struct FooVtable FooInstanceVtable_Float;
struct FooVtable FooInstanceVtable_Integer;
struct FooVtable FooInstanceVtable_String;
struct Foo foo_Float_new(double f);
struct Foo foo_Integer_new(int64_t n);
struct Foo foo_String_new(size_t len, const char* s);
struct Foo foo_vtable_typecheck(struct FooVtable* vtable, struct Foo obj);
struct Foo FooGlobal_True;
struct Foo FooGlobal_False;
struct FooContext* foo_context_new_block(struct FooContext* ctx);

struct FooContext* foo_context_new_main(struct FooArray* vars) {
  struct FooContext* context = foo_alloc_context(0);
  context->info = "main";
  context->receiver = FooGlobal_False;
  context->sender = NULL;
  context->outer_context = NULL;
  context->vars = vars;
  return context;
}

struct FooContext* foo_context_new_block(struct FooContext* sender) {
  struct FooBlock* block = sender->receiver.datum.ptr;
  struct FooContext* context = foo_alloc_context(block->frameSize);
  context->info = "block";
  context->receiver = block->context->receiver;
  context->sender = sender;
  context->outer_context = block->context;
  context->vars = sender->vars;
  for (size_t i = 0; i < block->argCount; ++i)
    context->frame[i] = sender->frame[i];
  return context;
}

struct FooContext* foo_context_new_unwind(struct FooContext* ctx, struct FooBlock* block) {
  struct FooContext* context = foo_alloc_context(block->frameSize);
  context->info = "#finally:";
  context->receiver = block->context->receiver;
  context->sender = ctx;
  context->outer_context = block->context;
  context->vars = ctx->vars;
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
  sender->vars->data[unbind->index] = unbind->value;
}

struct FooInterface {
  // struct FooVtable* instanceVtable;
  // struct FooVtable* classVtable;
};

struct FooVtableList {
  size_t size;
  struct FooVtable** data;
};

typedef void (*FooMarkFunction)(union FooDatum Foo);

struct FooVtable {
  struct FooCString* name;
  struct Foo* classptr;
  struct FooVtableList inherited;
  FooMarkFunction mark;
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
  struct FooVtableList* list = &obj.vtable->inherited;
  for (size_t i = 0; i < list->size; i++)
    if (vtable == list->data[i]) {
      return obj;
  }
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
  return_context->return_value = value;
  longjmp(*(jmp_buf*)return_context->ret, 1);
  FOO_PANIC("longjmp() fell through!")
}

struct FooContext* foo_context_new_method(const struct FooMethod* method,
                                          struct FooContext* sender,
                                          struct Foo receiver,
                                          size_t nargs, va_list arguments) {
  if (method->argCount != nargs) {
    FOO_PANIC("Wrong number of arguments to %s. Wanted: %zu, got: %zu.",
              method->selector->name->data, method->argCount, nargs);
  }
  if (method->frameSize < nargs) {
    FOO_PANIC("Method %s frame too small: %zu, got %zu arguments!",
              method->selector->name->data,
              method->frameSize,
              nargs);
  }
  assert(method->frameSize >= nargs);
  struct FooContext* context = foo_alloc_context(method->frameSize);
  context->info = method->selector->name->data;
  context->sender = sender;
  context->receiver = receiver;
  context->outer_context = NULL;
  context->vars = sender->vars;
  for (size_t i = 0; i < nargs; i++) {
    context->frame[i] = va_arg(arguments, struct Foo);
  }
  return context;
}

bool foo_eq(struct Foo a, struct Foo b) {
  return a.vtable == b.vtable && a.datum.int64 == b.datum.int64;
}

struct Foo foo_send(struct FooContext* sender,
                    const struct FooSelector* selector,
                    struct Foo receiver,
                    size_t nargs, ...) {
  FOO_DEBUG("/foo_send(?, %s, ...)", selector->name->data);
  va_list arguments;
  va_start(arguments, nargs);
  assert(receiver.vtable);
  const struct FooMethod* method = foo_vtable_find_method(receiver.vtable, selector);
  if (method) {
    struct FooContext* context = foo_context_new_method(method, sender, receiver, nargs, arguments);
    foo_maybe_gc(context);
    jmp_buf ret;
    context->ret = &ret;
    int jmp = setjmp(ret);
    if (jmp) {
      FOO_DEBUG("/foo_send -> non-local return from %s", selector->name->data);
      return context->return_value;
    } else {
      struct Foo res = method->function((struct FooContext*)context);
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
  struct FooBlock* block = foo_alloc(sizeof(struct FooBlock));
  block->context = context;
  block->function = function;
  block->argCount = argCount;
  block->frameSize = frameSize;
  return (struct Foo){ .vtable = &FooInstanceVtable_Block, .datum = { .ptr = block } };
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

struct FooArray* FooArray_alloc(size_t size) {
  struct FooArray* array = foo_alloc(sizeof(struct FooArray) + size*sizeof(struct Foo));
  array->size = size;
  return array;
}

struct Foo foo_Array_new(size_t size) {
  struct FooArray* array = FooArray_alloc(size);
  for (size_t i = 0; i < size; ++i) {
    array->data[i] = FooGlobal_False;
  }
  return (struct Foo){ .vtable = &FooInstanceVtable_Array, .datum = { .ptr = array } };
}

struct Foo foo_Array_alloc(size_t size) {
  struct FooArray* array = FooArray_alloc(size);
  return (struct Foo){ .vtable = &FooInstanceVtable_Array, .datum = { .ptr = array } };
}

struct Foo foo_Boolean_new(bool t) {
  return (struct Foo){ .vtable = &FooInstanceVtable_Boolean, .datum = { .boolean = t } };
}

struct Foo foo_Integer_new(int64_t n) {
  return (struct Foo){ .vtable = &FooInstanceVtable_Integer, .datum = { .int64 = n } };
}

struct Foo foo_Float_new(double f) {
  return (struct Foo){ .vtable = &FooInstanceVtable_Float, .datum = { .float64 = f } };
}


struct FooBytes* FooBytes_alloc(size_t len) {
  struct FooBytes* bytes = (struct FooBytes*)foo_alloc(sizeof(struct FooBytes) + len + 1);
  bytes->size = len;
  return bytes;
}

struct Foo foo_String_new(size_t len, const char* s) {
  struct FooBytes* bytes = FooBytes_alloc(len);
  memcpy(bytes->data, s, len);
  return (struct Foo) { .vtable = &FooInstanceVtable_String, .datum = { .ptr = bytes } };
}

struct Foo foo_String_new_from(const char* s) {
  return foo_String_new(strlen(s), s);
}

struct Foo foo_panic(struct Foo message) __attribute__((noreturn));
struct Foo foo_panic(struct Foo message) {
  struct FooBytes* bytes
    = PTR(FooBytes, foo_vtable_typecheck(&FooInstanceVtable_String, message).datum);
  printf("PANIC: %s", (char*)bytes->data);
  putchar('\n');
  fflush(stdout);
  fflush(stderr);
  _Exit(1);
}

void fooinit(void) {
#ifdef _WIN32
  _setmode(_fileno(stdout), O_BINARY);
  _setmode(_fileno(stderr), O_BINARY);
#endif
}

/**
   GC

 */

enum FooMark {
  RED = 0,
  BLUE = 1
};

static enum FooMark current_live_mark = RED;

void foo_flip_mark() {
  current_live_mark = !current_live_mark;
}

struct FooAlloc {
  struct FooAlloc* next;
  enum FooMark mark;
  size_t size;
  char data[];
};

bool foo_mark_ptr(void* ptr) {
  const size_t offset = offsetof(struct FooAlloc, data);
  struct FooAlloc* alloc = (void*)((char*)ptr-offset);
  bool new_mark = alloc->mark == current_live_mark;
  alloc->mark = current_live_mark;
  return new_mark;
}

void foo_mark_context(struct FooContext* ctx);

void foo_mark_object(struct Foo obj) {
  if (obj.vtable) {
    obj.vtable->mark(obj.datum);
  }
}

void foo_mark_noop(union FooDatum datum) {
  (void)datum;
}

void foo_mark_array(union FooDatum datum) {
  struct FooArray* array = PTR(FooArray, datum);
  if (foo_mark_ptr(array)) {
    for (size_t i = 0; i < array->size; i++) {
      foo_mark_object(array->data[i]);
    }
  }
}

void foo_mark_block(union FooDatum datum) {
  struct FooBlock* block = PTR(FooBlock, datum);
  if (foo_mark_ptr(block)) {
    foo_mark_context(block->context);
  }
}

void foo_mark_cleanup(struct FooCleanup* cleanup) {
  if (!cleanup) {
      return;
  }
  if (cleanup->function == foo_finally) {
    return foo_mark_block((union FooDatum){ .ptr = cleanup });
  }
  if (cleanup->function == foo_unbind) {
    return foo_mark_object(((struct FooUnbind*)cleanup)->value);
  }
  FOO_PANIC("invalid cleanup");
}

void foo_mark_context(struct FooContext* ctx) {
  if (!ctx) {
    return;
  }
  if (foo_mark_ptr(ctx)) {
    foo_mark_object(ctx->receiver);
    foo_mark_object(ctx->return_value);
    foo_mark_context(ctx->sender);
    foo_mark_context(ctx->outer_context);
    // vars are shared between all contexts, foo_mark() processes
    // them directly.
    foo_mark_cleanup(ctx->cleanup);
    for (size_t i = 0; i < ctx->size; i++) {
      foo_mark_object(ctx->frame[i]);
    }
  }
}

static struct FooAlloc* allocations = NULL;
static size_t allocation_count_since_gc = 0;
static size_t allocation_bytes_since_gc = 0;
static size_t allocation_bytes = 0;
static size_t allocation_count = 0;

// Intentionally low threshold so that GC gets exercised even for trivial tests.
const size_t gc_threshold = 512;
const bool gc_verbose = false;

void foo_sweep() {
  struct FooAlloc** tail = &allocations;
  struct FooAlloc* head = *tail;
  size_t freed_count = 0;
  size_t freed_bytes = 0;
  while (head) {
    struct FooAlloc* next = head->next;
    if (current_live_mark != head->mark) {
      *tail = next;
      freed_bytes += head->size;
      freed_count += 1;
      // free(head);
    }
    head = next;
  }
  if (freed_count > 0) {
    allocation_bytes -= freed_bytes;
    allocation_count -= freed_count;
    if (gc_verbose) {
      fprintf(stderr, "** GC'd %zu bytes in %zu objects, %zu bytes in %zu objects remain.\n",
              freed_bytes, freed_count,
              allocation_bytes, allocation_count);
      fprintf(stderr, "** %zu bytes in %zu objects allocated since last gc.\n",
              allocation_bytes_since_gc, allocation_count_since_gc);
    }
    allocation_count_since_gc = 0;
    allocation_bytes_since_gc = 0;
  }
}

void foo_maybe_gc(struct FooContext* ctx) {
  if (allocation_bytes_since_gc > gc_threshold) {
    foo_flip_mark();
    if (ctx->vars) {
      foo_mark_array((union FooDatum){ .ptr = ctx->vars });
    }
    foo_mark_context(ctx);
    foo_sweep();
  }
}

void* foo_alloc(size_t size) {
  size_t bytes = sizeof(struct FooAlloc) + size;
  struct FooAlloc* p = calloc(1, bytes);
  if (!p) {
    foo_abort("calloc");
  }
  p->next = allocations;
  p->size = bytes;
  p->mark = current_live_mark;
  allocations = p;

  allocation_bytes_since_gc += bytes;
  allocation_bytes += bytes;
  allocation_count_since_gc += 1;
  allocation_count += 1;

  return p->data;
}

#include "generated_declarations.h"
#include "generated_builtins.c"
#include "generated_constants.c"
#include "generated_blocks.c"
#include "generated_main.c"
