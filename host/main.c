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

struct FooContext;
void* foo_alloc(size_t bytes);
void foo_maybe_gc(struct FooContext* ctx);

struct Foo foo_panic(struct FooContext* ctx, struct Foo message) __attribute__((noreturn));
struct Foo foo_panicf(struct FooContext* ctx, const char* fmt, ...) __attribute__((noreturn));

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
 *  The big issue is that these cannot be passed out from selectors without
 *  copying. At least a vtable is needed...
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

enum FooContextType {
    METHOD_CONTEXT,
    BLOCK_CONTEXT,
    UNWIND_CONTEXT,
    ROOT_CONTEXT
};

struct FooContext {
  enum FooContextType type;
  uint32_t depth;
  const struct FooMethod* method;
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
  snprintf(s, size, "{ .type = %u, .size = %zu, .outer_context = %p }",
           ctx->type, ctx->size, ctx->outer_context);
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
  // The vtable in which this method originates.
  struct FooVtable* vtable;
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
struct Foo foo_vtable_typecheck(struct FooContext* ctx, struct FooVtable* vtable, struct Foo obj);
struct Foo FooGlobal_True;
struct Foo FooGlobal_False;
struct FooContext* foo_context_new_block(struct FooContext* ctx);

struct FooContext* foo_context_new_main(struct FooArray* vars) {
  struct FooContext* context = foo_alloc_context(0);
  context->type = ROOT_CONTEXT;
  context->depth = 0;
  context->method = NULL;
  context->receiver = FooGlobal_False;
  context->sender = NULL;
  context->outer_context = NULL;
  context->vars = vars;
  return context;
}

struct FooContext* foo_context_new_block(struct FooContext* sender) {
  struct FooBlock* block = sender->receiver.datum.ptr;
  struct FooContext* context = foo_alloc_context(block->frameSize);
  context->type = BLOCK_CONTEXT;
  context->depth = sender->depth + 1;
  context->method = NULL;
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
  context->type = UNWIND_CONTEXT;
  context->depth = ctx->depth + 1;
  context->method = NULL;
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

typedef void (*FooMarkFunction)(void* ptr);

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

struct Foo foo_vtable_typecheck(struct FooContext* ctx,
                                struct FooVtable* vtable,
                                struct Foo obj) {
  if (vtable == obj.vtable)
    return obj;
  struct FooVtableList* list = &obj.vtable->inherited;
  for (size_t i = 0; i < list->size; i++)
    if (vtable == list->data[i]) {
      return obj;
  }
  assert(vtable);
  assert(obj.vtable);
  foo_panicf(ctx, "Type error! Wanted: %s, got: %s",
             vtable->name->data, obj.vtable->name->data);
}

const struct FooMethod* foo_vtable_find_method(struct FooContext* ctx,
                                               const struct FooVtable* vtable,
                                               const struct FooSelector* selector) {
  assert(vtable);
  // FOO_DEBUG("/foo_vtable_find_method(%s#%s)", vtable->name->data, selector->name->data);
  for (size_t i = 0; i < vtable->size; ++i) {
    const struct FooMethod* method = &vtable->methods[i];
    if (method->selector == selector) {
      return method;
    }
  }
  foo_panicf(ctx, "%s does not understand: #%s", vtable->name->data, selector->name->data);
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
  foo_panicf(ctx, "INTERNAL ERROR: longjmp() fell through!");
}

struct FooContext* foo_context_new_method_no_args(const struct FooMethod* method,
                                                  struct FooContext* sender,
                                                  struct Foo receiver,
                                                  size_t nargs) {
  if (method->argCount != nargs) {
    foo_panicf(sender, "Wrong number of arguments to %s. Wanted: %zu, got: %zu.",
               method->selector->name->data, method->argCount, nargs);
  }
  if (method->frameSize < nargs) {
    foo_panicf(sender, "INTERNAL ERROR: Method %s frame too small: %zu, got %zu arguments!",
               method->selector->name->data,
               method->frameSize,
               nargs);
  }
  assert(method->frameSize >= nargs);
  struct FooContext* context = foo_alloc_context(method->frameSize);
  context->type = METHOD_CONTEXT;
  context->depth = sender->depth + 1;
  context->method = method;
  context->sender = sender;
  context->receiver = receiver;
  context->outer_context = NULL;
  context->vars = sender->vars;
  return context;
}

struct FooContext* foo_context_new_method_ptr(const struct FooMethod* method,
                                              struct FooContext* sender,
                                              struct Foo receiver,
                                              size_t nargs, struct Foo* arguments) {
  struct FooContext* context
    = foo_context_new_method_no_args(method, sender, receiver, nargs);
  for (size_t i = 0; i < nargs; i++) {
    context->frame[i] = arguments[i];
  }
  return context;
}

struct FooContext* foo_context_new_method_va(const struct FooMethod* method,
                                          struct FooContext* sender,
                                          struct Foo receiver,
                                          size_t nargs, va_list arguments) {
  struct FooContext* context
    = foo_context_new_method_no_args(method, sender, receiver, nargs);
  for (size_t i = 0; i < nargs; i++) {
    context->frame[i] = va_arg(arguments, struct Foo);
  }
  return context;
}

bool foo_eq(struct Foo a, struct Foo b) {
  return a.vtable == b.vtable && a.datum.int64 == b.datum.int64;
}

void foo_print_backtrace(struct FooContext* context) {
  printf("Backtrace:\n");
  struct FooVtable* home;
  struct FooVtable* here;
  while (context && context->depth) {
    switch (context->type) {
    case METHOD_CONTEXT:
      home = context->method->vtable;
      here = context->receiver.vtable;
      printf("  %u: ", context->depth);
      printf("%s#%s", home->name->data, context->method->selector->name->data);
      if (here != home) {
        printf(" (%s)", here->name->data);
      }
      printf("\n");
      break;
    case BLOCK_CONTEXT:
      // The method frame appears just before this one, not need to
      // print this separately. Even the frame numbers are right.
      break;
    case UNWIND_CONTEXT:
      printf("<<unwind>>");
      break;
    default:
      printf("<<unknown context type: %u>", context->type);
    }
    context = context->sender;
  }
}

struct Foo foo_activate(struct FooContext* context) {
  if (context->depth > 200) {
    foo_panicf(context, "Stack blew up!");
  }
  foo_maybe_gc(context);
  jmp_buf ret;
  context->ret = &ret;
  int jmp = setjmp(ret);
  if (jmp) {
    FOO_DEBUG("/foo_send -> non-local return from %s", selector->name->data);
    return context->return_value;
  } else {
    struct Foo res = context->method->function((struct FooContext*)context);
    FOO_DEBUG("/foo_send -> local return from %s", selector->name->data);
    return res;
  }

}

struct Foo foo_send_ptr(struct FooContext* sender,
                        const struct FooSelector* selector,
                        struct Foo receiver,
                        size_t nargs,
                        struct Foo* arguments) {
  FOO_DEBUG("/foo_send_ptr(?, %s, ...)", selector->name->data);
  assert(receiver.vtable);
  const struct FooMethod* method
    = foo_vtable_find_method(sender, receiver.vtable, selector);
  struct FooContext* context
    = foo_context_new_method_ptr(method, sender, receiver, nargs, arguments);
  return foo_activate(context);
}

struct Foo foo_send(struct FooContext* sender,
                    const struct FooSelector* selector,
                    struct Foo receiver,
                    size_t nargs, ...) {
  FOO_DEBUG("/foo_send(?, %s, ...)", selector->name->data);
  va_list arguments;
  va_start(arguments, nargs);
  assert(receiver.vtable);
  const struct FooMethod* method
    = foo_vtable_find_method(sender, receiver.vtable, selector);
  struct FooContext* context
    = foo_context_new_method_va(method, sender, receiver, nargs, arguments);
  return foo_activate(context);
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

struct Foo foo_panic(struct FooContext* ctx, struct Foo message) {
  printf("PANIC: ");
  if (&FooInstanceVtable_String == message.vtable) {
    struct FooBytes* bytes = PTR(FooBytes, message.datum);
    printf("%s", (char*)bytes->data);
  } else {
    printf("<<cannot print panic reason: not a String>>");
  }
  printf("\n");
  foo_print_backtrace(ctx);
  fflush(stdout);
  fflush(stderr);
  _Exit(1);
}

struct Foo foo_panicf(struct FooContext* ctx, const char* fmt, ...) {
  va_list arguments;
  va_start(arguments, fmt);
  printf("PANIC: ");
  vprintf(fmt, arguments);
  printf("\n");
  foo_print_backtrace(ctx);
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

#if 0
#define DEBUG_GC(...) { printf(__VA_ARGS__); fflush(stdout); }
#else
#define DEBUG_GC(...)
#endif

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

bool foo_mark_live(void* ptr) {
  const size_t offset = offsetof(struct FooAlloc, data);
  struct FooAlloc* alloc = (void*)((char*)ptr-offset);
  bool new_mark = alloc->mark != current_live_mark;
  DEBUG_GC("    /mark_live %p mark=%d, live=%d\n", ptr, alloc->mark, current_live_mark);
  alloc->mark = current_live_mark;
  return new_mark;
}

void foo_mark_ptr(void* ptr) {
  foo_mark_live(ptr);
}

void foo_mark_context(struct FooContext* ctx);

void foo_mark_object(struct Foo obj) {
  if (obj.vtable) {
    obj.vtable->mark(obj.datum.ptr);
  }
}

void foo_mark_raw(void* ptr) {
  (void)ptr;
  DEBUG_GC("  /mark_raw\n");
}

void foo_mark_static(void* ptr) {
  (void)ptr;
  DEBUG_GC("  /mark_static\n");
}

void foo_mark_array(void* ptr) {
  DEBUG_GC("  /mark_array\n");
  struct FooArray* array = ptr;
  if (foo_mark_live(array)) {
    for (size_t i = 0; i < array->size; i++) {
      foo_mark_object(array->data[i]);
    }
  }
}

void foo_mark_block(void* ptr) {
  DEBUG_GC("  /mark_block\n");
  struct FooBlock* block = ptr;
  if (foo_mark_live(block)) {
    foo_mark_context(block->context);
  }
}

void foo_mark_cleanup(struct FooCleanup* cleanup) {
  DEBUG_GC("  /mark_cleanup\n");
  if (!cleanup) {
      return;
  }
  if (cleanup->function == foo_finally) {
    return foo_mark_block(((struct FooFinally*)cleanup)->block);
  }
  if (cleanup->function == foo_unbind) {
    return foo_mark_object(((struct FooUnbind*)cleanup)->value);
  }
}

void foo_mark_context(struct FooContext* ctx) {
  if (!ctx) {
    return;
  }
  DEBUG_GC("/mark_context\n");
  if (foo_mark_live(ctx)) {
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
      free(head);
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
    DEBUG_GC("--GC--\n");
    foo_flip_mark();
    if (ctx->vars) {
      foo_mark_array(ctx->vars);
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
#include "generated_constants.c"
#include "generated_builtins.c"
#include "generated_blocks.c"
#include "generated_main.c"
