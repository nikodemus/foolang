#include "mark-and-sweep.h"
#include "foo.h"

#include <stdlib.h>
#include <stdio.h>
#include <stddef.h>
#include <assert.h>

bool trace_gc = false;
bool identify_gc_epoch = false;

size_t gc_trace_depth = 0;
size_t gc_epoch = 0;

size_t gc_trace_start_epoch = 0;
size_t gc_trace_end_epoch = 0;

size_t secondary_gc_epoch_start = 0;
size_t secondary_gc_epoch_end = 0;

static inline size_t zmin(size_t a, size_t b) {
  if (a <= b)
    return a;
  else
    return b;
}
#define ENTER_TRACE(...)                                                \
  if (trace_gc) {                                                       \
    fprintf(stderr, "\n");                                              \
    for (size_t i = 0; i < zmin(120,gc_trace_depth); i++)               \
      fprintf(stderr, " ");                                             \
    fprintf(stderr, "%zu: ", gc_trace_depth);                           \
    fprintf(stderr, __VA_ARGS__);                                       \
    fflush(stderr);                                                     \
    gc_trace_depth++;                                                   \
  }                                                                     \

#define EXIT_TRACE()                                                    \
  if (trace_gc) {                                                       \
    gc_trace_depth--;                                                   \
    if (!gc_trace_depth)                                                \
      fprintf(stderr, "\n");                                            \
  }                                                                     \

#define EXIT_TRACE_VERBOSE(...)                                         \
  if (trace_gc) {                                                       \
    gc_trace_depth--;                                                   \
    fprintf(stderr, "\n");                                              \
    for (size_t i = 0; i < zmin(120,gc_trace_depth); i++)               \
      fprintf(stderr, " ");                                             \
    fprintf(stderr, "%zu: ", gc_trace_depth);                           \
    fprintf(stderr, __VA_ARGS__);                                       \
    if (!gc_trace_depth)                                                \
      fprintf(stderr, "\n");                                            \
    fflush(stderr);                                                     \
  }                                                                     \

#if 0
#define DEBUG_GC(...) if (trace_gc) { fprintf(stderr, __VA_ARGS__); fflush(stderr); }
#else
#define DEBUG_GC(...)
#endif

enum FooMark {
  DEAD = 0,
  LIVE = 1,
};

struct FooAlloc {
  enum FooMark mark;
  struct FooAlloc* next;
  size_t size;
  char data[];
};

bool foo_mark_live(void* ptr) {
  ENTER_TRACE("mark_live %p", ptr);
  const size_t offset = offsetof(struct FooAlloc, data);
  struct FooAlloc* alloc = (void*)((char*)ptr-offset);
  bool new_mark = alloc->mark == DEAD;
  alloc->mark = LIVE;
  EXIT_TRACE();
  return new_mark;
}

void foo_mark_ptr(void* ptr) {
  ENTER_TRACE("mark_ptr");
  foo_mark_live(ptr);
  EXIT_TRACE();
}

void foo_mark_bytes(void* ptr) {
  ENTER_TRACE("mark_bytes");
  struct FooBytes* bytes = ptr;
  if (bytes->header.allocation == HEAP) {
    foo_mark_live(ptr);
  }
  EXIT_TRACE();
}

void foo_mark_file(void* ptr) {
  ENTER_TRACE("mark_file");
  struct FooFile* file = ptr;
  if (file->header.allocation == HEAP && foo_mark_live(file)) {
    foo_mark_bytes(file->pathname);
  }
  EXIT_TRACE();
}

void foo_mark_filestream(void* ptr) {
  ENTER_TRACE("mark_bytes");
  struct FooFileStream* stream = ptr;
  if (stream->header.allocation == HEAP && foo_mark_live(stream))  {
    foo_mark_bytes(stream->pathname);
  }
  EXIT_TRACE();
}

void foo_mark_context(struct FooContext* ctx);

void foo_mark_object(struct Foo obj) {
  ENTER_TRACE("mark_object datum=%p, class=%p", obj.datum.ptr, obj.class);
  if (obj.class) {
    DEBUG_GC(" (%s)", obj.class->name->data);
    foo_mark_class(obj.class);
    obj.class->mark(obj.datum.ptr);
  } else {
    assert(!obj.datum.int64);
  }
  EXIT_TRACE();
}

void foo_mark_none(void* ptr) {
  (void)ptr;
}

void foo_mark_oops(void* ptr) {
  foo_abort("Oops");
}

void foo_mark_array(void* ptr) {
  ENTER_TRACE("mark_array");
  struct FooArray* array = ptr;
  if (array->header.allocation == HEAP && foo_mark_live(array)) {
    for (size_t i = 0; i < array->size; i++) {
      foo_mark_object(array->data[i]);
    }
  }
  EXIT_TRACE();
}

void foo_mark_instance(void* ptr) {
  ENTER_TRACE("mark_instance");
  struct FooArray* array = ptr;
  if (array->header.allocation == HEAP && foo_mark_live(array)) {
    for (size_t i = 0; i < array->size; i++) {
      foo_mark_object(array->data[i]);
    }
  }
  EXIT_TRACE();
}

void foo_mark_layout(void* ptr) {
  struct FooLayout* layout = ptr;
  bool is_empty = layout == &TheEmptyLayout;
  bool is_class = layout == &TheClassLayout;
  (void)is_empty;
  (void)is_class;
  ENTER_TRACE("mark_layout (%s)",
              is_empty ? "empty" : (is_class ? "class" : "object"));
  if (layout->header.allocation == HEAP) {
    foo_mark_live(layout);
  }
  EXIT_TRACE();
}

void foo_mark_class_list(void* ptr) {
  struct FooClassList* list = ptr;
  ENTER_TRACE("mark_class_list %p (size=%zu)", list, list->size);
  if (list->header.allocation == HEAP && foo_mark_live(list)) {
    for (size_t i = 0; i < list->size; i++) {
      foo_mark_class(list->data[i]);
    }
  }
  EXIT_TRACE();
}

void foo_mark_class(void* ptr)
{
  struct FooClass* class = ptr;
  ENTER_TRACE("mark_class %p", class);
  DEBUG_GC(" name = %p", class->name);
  DEBUG_GC(" (%s)", class->name->data);
  assert(class);
  if (class->header.allocation == HEAP && foo_mark_live(class)) {
    foo_mark_bytes(class->name);
    foo_mark_class(class->metaclass);
    foo_mark_class_list(class->inherited);
    foo_mark_layout(class->layout);
    for (size_t i = 0; i < class->inherited->size; i++) {
      struct FooClass* other = class->inherited->data[i];
      if (other)
        foo_mark_class(other);
    }
    for (size_t i = 0; i < class->size; i++) {
      struct FooMethod* method = &class->methods[i];
      if (method->object.class)
        foo_mark_object(method->object);
    }
  }
  EXIT_TRACE();
}

void foo_mark_closure(void* ptr) {
  ENTER_TRACE("mark_closure");
  struct FooClosure* closure = ptr;
  if (foo_mark_live(closure)) {
    foo_mark_context(closure->context);
  }
  EXIT_TRACE();
}

void foo_mark_cleanup(struct FooCleanup* cleanup) {
  ENTER_TRACE("mark_cleanup");
  if (!cleanup) {
    goto exit;
  }
  if (cleanup->function == foo_finally) {
    foo_mark_closure(((struct FooFinally*)cleanup)->closure);
    goto exit;
  }
  if (cleanup->function == foo_unbind) {
    foo_mark_object(((struct FooUnbind*)cleanup)->value);
    goto exit;
  }
 exit:
  EXIT_TRACE();
}

void foo_mark_context(struct FooContext* ctx) {
  ENTER_TRACE("mark_context %p", ctx);
  if (ctx) {
    DEBUG_GC(" type=%d, alloc=%d", ctx->type, ctx->header.allocation);
    DEBUG_GC(" depth: %u, size: %zu", ctx->depth, ctx->size);
    if (ctx->type == METHOD_CONTEXT) {
      DEBUG_GC(" selector: %s", ctx->method->selector->name->data);
    }
  }
  if (ctx &&
      (ctx->header.allocation == STACK ||
       (ctx->header.allocation == HEAP && foo_mark_live(ctx)))) {
    // Stack allocated contexts get traced but not marked.
    foo_mark_object(ctx->receiver);
    foo_mark_context(ctx->sender);
    foo_mark_context(ctx->outer_context);
    // vars are shared between all contexts, foo_mark() processes them directly.
    foo_mark_cleanup(ctx->cleanup);
    foo_mark_object(ctx->return_value);
    for (size_t i = 0; i < ctx->size; i++) {
      // printf("\n[%zu]", i);
      foo_mark_object(ctx->frame[i]);
    }
  }
  EXIT_TRACE();
}

struct FooAlloc* allocations = NULL;
static size_t allocation_count_since_gc = 0;
static size_t allocation_bytes_since_gc = 0;
static size_t allocation_bytes = 0;
static size_t allocation_count = 0;

// Intentionally low threshold so that GC gets exercised even for trivial tests.
// const size_t gc_threshold = 1024;
const size_t gc_threshold = 1024 * 1024 * 64;
bool gc_verbose = false;

void foo_sweep() {
  struct FooAlloc* head = allocations;
  size_t freed_count = 0, freed_bytes = 0;
  size_t live_count = 0, live_bytes = 0;

  size_t n = 0;
  struct FooAlloc* prev = NULL;
  while (head) {
    n++;
    struct FooAlloc* next = head->next;
    if (head->mark == DEAD) {
      if (!prev) {
        allocations = next;
      } else {
        prev->next = next;
      }
      freed_bytes += head->size;
      freed_count += 1;
      free(head);
    } else {
      prev = head;
      live_count += 1;
      live_bytes += head->size;
    }
    head = next;
  }

  assert(allocation_count == n);

  if (freed_count > 0) {
    allocation_bytes -= freed_bytes;
    allocation_count -= freed_count;

    if (gc_verbose) {
      double mb = 1024.0 * 1024.0;
      fprintf(stderr,
              "-- %.2fMB in %zu objects allocated since last gc\n"
              "-- %.2fMB in %zu objects collected\n"
              "-- %.2fMB in %zu objects remain\n",
              allocation_bytes_since_gc / mb, allocation_count_since_gc,
              freed_bytes / mb, freed_count,
              allocation_bytes / mb, allocation_count);
      fflush(stderr);
    }

    assert(live_count == allocation_count);
    assert(live_bytes == allocation_bytes);

    allocation_count_since_gc = 0;
    allocation_bytes_since_gc = 0;
  }
}

void foo_gc_impl(struct FooContext* ctx) {
  // Mark everything dead
  struct FooAlloc* head = allocations;
  while (head) {
    head->mark = DEAD;
    head = head->next;
  }

  // Mark everything from ctx live
  if (ctx->vars) {
    ENTER_TRACE("mark root vars");
    foo_mark_array(ctx->vars);
    EXIT_TRACE();
  }
  ENTER_TRACE("mark root context");
  foo_mark_context(ctx);
  EXIT_TRACE();

  // Free dead things
  ENTER_TRACE("sweep");
  foo_sweep();
  EXIT_TRACE();
}

void foo_gc(struct FooContext* ctx) {
    ++gc_epoch;

    // Check if we're in an epoch we want to trace.
    bool prev_trace = trace_gc;
    if (gc_trace_start_epoch <= gc_epoch && gc_epoch <= gc_trace_end_epoch) {
      trace_gc = true;
    }
    if (!trace_gc && identify_gc_epoch) {
      fprintf(stderr, "GC (epoch=%zu)\n", gc_epoch);
    }

    ENTER_TRACE("Primary GC (epoch=%zu)\n", gc_epoch);
    foo_gc_impl(ctx);
    EXIT_TRACE_VERBOSE("Primary GC (epoch=%zu) complete\n", gc_epoch);
    
    if (secondary_gc_epoch_start <= gc_epoch && gc_epoch <= secondary_gc_epoch_end) {
      ENTER_TRACE("secondary GC (epoch=%zu)", gc_epoch);
      foo_gc_impl(ctx);
      EXIT_TRACE_VERBOSE("secondary GC (epoch=%zu) complete", gc_epoch);
    }

    trace_gc = prev_trace;
}

void* foo_alloc(struct FooContext* sender, size_t size) {
  if (allocation_bytes_since_gc > gc_threshold && sender) {
    foo_gc(sender);
  }
  size_t bytes = sizeof(struct FooAlloc) + size;
  struct FooAlloc* p = calloc(1, bytes);
  if (!p) {
    foo_panicf(sender, "Allocation failed. Tried to allocate %zu bytes, total allocated %zu bytes.",
               bytes, allocation_bytes);
  }
  p->next = allocations;
  p->size = bytes;
  p->mark = LIVE;
  allocations = p;

  allocation_bytes_since_gc += bytes;
  allocation_bytes += bytes;
  allocation_count_since_gc += 1;
  allocation_count += 1;

  return p->data;
}

void* foo_alloc_no_gc(struct FooContext* sender, size_t size) {
  size_t bytes = sizeof(struct FooAlloc) + size;
  struct FooAlloc* p = calloc(1, bytes);
  if (!p) {
    foo_panicf(sender, "Allocation failed. Tried to allocate %zu bytes, total allocated %zu bytes.",
               bytes, allocation_bytes);
  }
  p->next = allocations;
  p->size = bytes;
  p->mark = LIVE;
  allocations = p;

  allocation_bytes_since_gc += bytes;
  allocation_bytes += bytes;
  allocation_count_since_gc += 1;
  allocation_count += 1;

  return p->data;
}
