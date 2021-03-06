// FIXME: fopen() on Windows, should use fopen_s instead.
#define _CRT_SECURE_NO_WARNINGS 1

#include <ctype.h>
#include <math.h>
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
#include <sys/stat.h>

#undef NDEBUG
#include <assert.h>

#ifdef _WIN32

#include <io.h>
#include <fcntl.h>
#define sys_stat _stat
#define sys_access _access
#define SYS_ISDIR(s) (_S_IFDIR & s)
#define SYS_ISREG(s) (_S_IFREG & s)

#else

#include <unistd.h>
#include <sys/types.h>
#include <fcntl.h>
#define sys_stat stat
#define sys_access access
#define SYS_ISDIR S_ISDIR
#define SYS_ISREG S_ISREG

#endif

#include "foo.h"
#include "system.h"
#include "ext.h"

size_t min_size(size_t a, size_t b) {
  if (a <= b) {
    return a;
  } else {
    return b;
  }
}

#define PTR(type, datum) \
  ((struct type*)datum.ptr)

#if 0
# define FOO_DEBUG(...) { fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); fflush(stderr); }
#else
# define FOO_DEBUG(...)
#endif

#define FOO_XXX(...) { fprintf(stderr, "XXX: "); fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); fflush(stderr); }

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
  struct FooClass* class;
  union FooDatum datum;
};

enum FooAllocType {
  STATIC,
  HEAP
};

struct FooHeader {
  enum FooAllocType allocation;
};

struct FooBytes {
  struct FooHeader header;
  size_t size;
  uint8_t data[];
};

#define FOO_CSTRING(literal) \
  ((struct FooBytes*) \
  &((struct { struct FooHeader header; size_t size; const char data[sizeof(literal)]; }) \
     { .header = { .allocation = STATIC }, .size = sizeof(literal)-1, .data = literal }))

bool foo_bytes_equal(const struct FooBytes* a, const struct FooBytes* b) {
  return a->size == b->size && !memcmp(a->data, b->data, a->size);
}

/** Simple intrusive list for interning. O(N), but fine to start with.
 */
struct FooSelector {
  struct FooSelector* next;
  struct FooBytes* name;
};

#include "generated_selectors.h"

struct FooSelector* foo_intern_new_selector(struct FooBytes* name) {
  // Need to copy the name: list of selectors isn't scanned by
  // GC, so if this becomes the only reference the memory would
  // be released.
  struct FooBytes* nameCopy = calloc(1, sizeof(struct FooBytes) + name->size + 1);
  memcpy(nameCopy->data, name->data, name->size);
  nameCopy->header.allocation = STATIC;
  nameCopy->size = name->size;
  struct FooSelector* new = calloc(1, sizeof(struct FooSelector));
  new->name = nameCopy;
  new->next = FOO_InternedSelectors;
  FOO_InternedSelectors = new;
  return new;
}

struct FooSelector* foo_intern(struct FooBytes* name) {
  struct FooSelector* selector = FOO_InternedSelectors;
  while (selector != NULL) {
    if (foo_bytes_equal(selector->name, name)) {
      return selector;
    } else {
      selector = selector->next;
    }
  }
  return foo_intern_new_selector(name);
}

struct FooArray {
  struct FooHeader header;
  size_t size;
  struct Foo data[];
};

// FIXME: Don't like defining this in C.
struct FooFile {
  struct FooHeader header;
  struct FooBytes* pathname;
  size_t mode;
};

// FIXME: Don't like defining this in C.
struct FooFileStream {
  struct FooHeader header;
  struct FooBytes* pathname;
  FILE* ptr;
};

struct FooCleanup;
typedef void (*FooCleanupFunction)(struct FooContext*, struct FooCleanup*);

struct FooCleanup {
  FooCleanupFunction function;
  struct FooCleanup* next;
};

struct FooFinally {
  struct FooCleanup cleanup;
  struct FooClosure* closure;
};

struct FooUnbind {
  struct FooCleanup cleanup;
  size_t index;
  struct Foo value;
};

enum FooContextType {
    METHOD_CONTEXT,
    CLOSURE_CONTEXT,
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

typedef struct Foo (*FooMethodFunction)(const struct FooMethod*, struct FooContext*);
typedef struct Foo (*FooClosureFunction)(struct FooContext*);

struct Foo foo_lexical_ref(struct FooContext* context, size_t index, size_t frameOffset) {
  struct FooContext* context0 = context;
  size_t frameOffset0 = frameOffset;
  // FOO_DEBUG("/lexical_ref(index=%zu, frame=%zu)", index, frameOffset);
  while (frameOffset > 0) {
    assert(context->outer_context);
    context = context->outer_context;
    --frameOffset;
  }
  assert(index < context->size);
  struct Foo res = context->frame[index];
  if (!res.class) {
    foo_panicf(context0, "Invalid lexical reference at index: %zu, frameOffset: %zu",
               index, frameOffset0);
  }
  return res;
}

struct Foo foo_lexical_set(struct FooContext* context, size_t index, size_t frame, struct Foo value) {
  // FOO_DEBUG("/lexical_set(index=%zu, frame=%zu, ...)", index, frame);
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
  struct FooClass* class; // FIXME: rename to home
  struct FooSelector* selector;
  size_t argCount;
  size_t frameSize;
  // Native method functions directly implement the method
  // Object method functions send #invoke:inContext: to the object
  FooMethodFunction function;
  struct Foo object;
};

struct FooClosure {
  struct FooContext* context;
  size_t argCount;
  size_t frameSize;
  FooClosureFunction function;
};

// Forward declarations for classs are in generated_classes, but we're going
// to define a few builtin ctors first that need some of them.
struct FooClass FooClass_Array;
struct FooClass FooClass_Character;
struct FooClass FooClass_Class;
struct FooClass FooClass_Closure;
struct FooClass FooClass_Boolean;
struct FooClass FooClass_Float;
struct FooClass FooClass_Integer;
struct FooClass FooClass_Selector;
struct FooClass FooClass_String;
struct FooPointerList FooClassInheritance_Class;
struct FooArray* FooArray_alloc(size_t size);
struct FooArray* FooArray_instance(size_t size);
struct Foo foo_Float_new(double f);
struct Foo foo_String_new(size_t len, const char* s);
struct Foo foo_class_typecheck(struct FooContext* ctx, struct FooClass* class, struct Foo obj);
struct Foo FooGlobal_True;
struct Foo FooGlobal_False;
struct FooContext* foo_context_new_closure(struct FooContext* ctx);

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

struct FooContext* foo_context_new_closure(struct FooContext* sender) {
  struct FooClosure* closure = sender->receiver.datum.ptr;
  struct FooContext* context = foo_alloc_context(closure->frameSize);
  context->type = CLOSURE_CONTEXT;
  context->depth = sender->depth + 1;
  context->method = NULL;
  context->receiver = closure->context->receiver;
  context->sender = sender;
  context->outer_context = closure->context;
  context->vars = sender->vars;
  for (size_t i = 0; i < closure->argCount; ++i)
    context->frame[i] = sender->frame[i];
  return context;
}

struct FooContext* foo_context_new_closure_array(struct FooContext* sender,
                                                 struct FooArray* array) {
  struct FooClosure* closure = sender->receiver.datum.ptr;
  struct FooContext* context = foo_alloc_context(closure->frameSize);
  context->type = CLOSURE_CONTEXT;
  context->depth = sender->depth + 1;
  context->method = NULL;
  context->receiver = closure->context->receiver;
  context->sender = sender;
  context->outer_context = closure->context;
  context->vars = sender->vars;
  assert(array->size == closure->argCount);
  for (size_t i = 0; i < closure->argCount; ++i)
    context->frame[i] = array->data[i];
  return context;
}

struct FooContext* foo_context_new_unwind(struct FooContext* ctx, struct FooClosure* closure) {
  struct FooContext* context = foo_alloc_context(closure->frameSize);
  context->type = UNWIND_CONTEXT;
  context->depth = ctx->depth + 1;
  context->method = NULL;
  context->receiver = closure->context->receiver;
  context->sender = ctx;
  context->outer_context = closure->context;
  context->vars = ctx->vars;
  assert(closure->argCount == 0);
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
  struct FooClosure* closure = ((struct FooFinally*)cleanup)->closure;
  // FIXME: Could stack-allocate this context.
  struct FooContext* closure_ctx = foo_context_new_unwind(sender, closure);
  closure->function(closure_ctx);
}

void foo_unbind(struct FooContext* sender, struct FooCleanup* cleanup) {
  struct FooUnbind* unbind = (struct FooUnbind*)cleanup;
  sender->vars->data[unbind->index] = unbind->value;
}

typedef void (*FooMarkFunction)(void* ptr);
void foo_mark_array(void* ptr);
void foo_mark_class(void* ptr);
void foo_mark_none(void* ptr);

struct FooPointerList {
  struct FooHeader header;
  FooMarkFunction mark;
  size_t size;
  void* data[];
};

struct FooPointerList* foo_ClassList_alloc(size_t size) {
  struct FooPointerList* list
    = foo_alloc(sizeof(struct FooPointerList)
                + size * sizeof(void*));
  list->header.allocation = HEAP;
  list->mark = foo_mark_class;
  list->size = size;
  return list;
}

struct FooLayout {
  struct FooHeader header;
  FooMarkFunction mark;
  size_t size;
};

struct FooLayout TheEmptyLayout = {
  .header = { .allocation = STATIC },
  .mark = foo_mark_none,
  .size = 0
};

struct FooLayout TheClassLayout = {
  .header = { .allocation = STATIC },
  .mark = foo_mark_none,
  .size = 0
};

struct FooLayout* foo_FooLayout_new(size_t size) {
  // Empty layout can be shared, everything else needs to the unique: otherwise
  // having layout to one class would allow direct access to instance variables
  // of another class instance with shape.
  //
  // (The class layout is a special case, but user code should not be able
  // to access it.)
  if (size == 0)
    return &TheEmptyLayout;
  struct FooLayout* layout = foo_alloc(sizeof(struct FooLayout));
  layout->header.allocation = HEAP;
  layout->mark = foo_mark_array;
  layout->size = size;
  return layout;
}

struct FooClass {
  struct FooHeader header;
  struct FooBytes* name;
  struct FooClass* metaclass;
  struct FooPointerList* inherited;
  struct FooLayout* layout;
  FooMarkFunction mark;
  size_t size;
  struct FooMethod methods[];
};

struct Foo foo_class_new(struct FooContext* ctx) {
  struct FooClass* theClass = PTR(FooClass, ctx->frame[0].datum);
  struct FooLayout* theLayout = PTR(FooLayout, ctx->receiver.datum);
  if (theClass->layout != theLayout) {
    foo_panicf(ctx, "Layout mismatch: invalid layout for %s",
               theClass->name->data);
  }
  if (theLayout->size != ctx->size - 1) {
    foo_panicf(ctx, "Layout mismatch: %s layout has %zu slots, using %zu slot constructor.",
               theClass->name->data, theLayout->size, ctx->size - 1);
  }
  struct FooArray* new = FooArray_alloc(theLayout->size);
  for (size_t i = 0; i < theLayout->size; i++) {
    new->data[i] = ctx->frame[i+1];
  }
  return (struct Foo){ .class = theClass,
                       .datum = { .ptr = new }};
}

struct Foo foo_class_new_from_array(struct FooContext* ctx) {
  struct FooClass* theClass = PTR(FooClass, ctx->frame[0].datum);
  struct FooLayout* theLayout = PTR(FooLayout, ctx->receiver.datum);
  if (theClass->layout != theLayout) {
    foo_panicf(ctx, "Layout mismatch: invalid layout for %s",
               theClass->name->data);
  }
  struct FooArray* theArray = PTR(FooArray, ctx->frame[1].datum);
  if (theLayout->size != theArray->size) {
    foo_panicf(ctx, "Layout mismatch: %s layout has %zu slots, array has %zu elements.",
               theClass->name->data, theLayout->size, theArray->size);
  }
  return (struct Foo){ .class = theClass,
                       .datum = { .ptr = theArray }};
}

bool foo_class_inherits(struct FooClass* want, struct FooClass* class) {
  struct FooPointerList* list = class->inherited;
  for (size_t i = 0; i < list->size; i++) {
    struct FooClass* ptr = list->data[i];
    if (want == ptr || foo_class_inherits(want, ptr)) {
      return true;
    }
  }
  return false;
}

struct Foo foo_class_typecheck(struct FooContext* ctx,
                               struct FooClass* class,
                               struct Foo obj) {
  assert(class);
  if (!obj.class) {
    foo_panicf(ctx, "Object has no class to check: %p, wanted %s",
               obj.datum.ptr, class->name->data);
  }
  if (class == obj.class || foo_class_inherits(class, obj.class)) {
    return obj;
  }
  foo_panicf(ctx, "Type error! Wanted: %s, got: %s",
             class->name->data, obj.class->name->data);
}

union FooDatum foo_check_modification(struct FooContext* ctx, union FooDatum datum) {
  if (((struct FooHeader*)datum.ptr)->allocation == STATIC) {
    foo_panicf(ctx, "Cannot modify constant object!");
  }
  return datum;
}

struct Foo foo_class_includes(struct FooContext* ctx,
                               struct FooClass* class,
                               struct Foo obj) {
  assert(class);
  assert(obj.class);
  return FOO_BOOLEAN(class == obj.class || foo_class_inherits(class, obj.class));
}

const struct FooMethod* foo_class_find_method_in(const struct FooClass* class,
                                                 const struct FooSelector* selector,
                                                 const struct FooMethod** fallback)
{
  const bool trace = false;
  if (trace) {
    assert(class);
    assert(class->name);
    assert(selector);
    assert(selector->name);
    FOO_XXX("foo_class_find_method_in(%s, #%s)",
            class->name->data, selector->name->data);
  }
  for (size_t i = 0; i < class->size; ++i) {
    const struct FooMethod* method = &class->methods[i];
    if (trace)
      FOO_XXX("  ? %s", method->selector->name->data);
    if (method->selector == selector) {
      if (trace)
        FOO_XXX("  #%s found!", selector->name->data);
      return method;
    } else if (!*fallback && method->selector == &FOO_perform_with_) {
      if (trace)
        FOO_XXX("  #perform:with: found! (fallback)");
      *fallback = method;
    }
  }
  return NULL;
}

struct FooProcessTimes FooFindMethodProfile = {
  .user = 0.0,
  .system = 0.0,
  .real = 0.0
};
size_t FooFindMethodCount = 0;

const struct FooMethod* foo_class_find_method(struct FooContext* ctx,
                                              const struct FooClass* class,
                                              const struct FooSelector* selector) {
  struct FooProcessTimes t0;
  const bool time_find_method = false;
  if (time_find_method) {
    system_get_process_times(&t0);
    assert(class);
  }
  const struct FooMethod* fallback = NULL;
  const struct FooMethod* method = foo_class_find_method_in(class, selector, &fallback);
  if (method) {
    goto found;
  }
  const struct FooPointerList* list = class->inherited;
  for (size_t i = 0; i < list->size; i++) {
    method = foo_class_find_method_in(list->data[i], selector, &fallback);
    if (method) {
      goto found;
    }
  }
  if (fallback) {
    method = fallback;
    goto found;
  }

  if (false) {
    for (size_t i = 0; i < class->size; i++) {
      const struct FooMethod* method = &class->methods[i];
      printf("- %s\n", method->selector->name->data);
    }
  }
  foo_panicf(ctx, "Instances of %s do not understand: #%s", class->name->data, selector->name->data);

 found:

  if (time_find_method) {
    struct FooProcessTimes t1;
    system_get_process_times(&t1);
    FooFindMethodProfile.user += t1.user - t0.user;
    FooFindMethodProfile.system += t1.system - t0.system;
    FooFindMethodProfile.real += t1.real - t0.real;
    if (++FooFindMethodCount % 1000 == 0) {
      FOO_XXX("find_method user: %f, system: %f, real: %f (%zu)",
              FooFindMethodProfile.user,
              FooFindMethodProfile.system,
              FooFindMethodProfile.real,
              FooFindMethodCount);
    }
  }

  return method;
}

struct Foo foo_return(struct FooContext* ctx, struct Foo value) __attribute__ ((noreturn));
struct Foo foo_return(struct FooContext* ctx, struct Foo value) {
  // FOO_DEBUG("/foo_return(%s...)", foo_debug_context(ctx));
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

void foo_check_method_argcount(struct FooContext* sender,
                               size_t nargs,
                               const struct FooMethod* method) {
  if (nargs != method->argCount) {
    foo_panicf(sender, "Wrong number of arguments! %s requires %zu, got %zu",
               method->selector->name->data, method->argCount, nargs);
  }
  if (method->frameSize < nargs) {
    foo_panicf(sender, "INTERNAL ERROR: Method %s frame too small: %zu, got %zu arguments!",
               method->selector->name->data,
               method->frameSize,
               nargs);
  }
}

struct FooContext* foo_context_new_method_no_args(const struct FooMethod* method,
                                                  struct FooContext* sender,
                                                  struct Foo receiver,
                                                  size_t nargs) {
  foo_check_method_argcount(sender, nargs, method);
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

struct FooContext* foo_context_new_method_array(const struct FooMethod* method,
                                                struct FooContext* sender,
                                                const struct FooSelector* selector,
                                                struct Foo receiver,
                                                struct Foo arguments) {
  struct FooContext* context
    = foo_context_new_method_no_args(method, sender, receiver, method->argCount);
  if (selector != method->selector) {
    // DoesNotUnderstand case
    assert(&FOO_perform_with_ == method->selector);
    assert(method->argCount == 2);
    context->frame[0] = (struct Foo){ .class = &FooClass_Selector,
                                      .datum = { .ptr = (void*)selector } };
    context->frame[1] = arguments;
  } else {
    // normal case
    struct FooArray* array = PTR(FooArray, arguments.datum);
    if (array->size != method->argCount) {
      foo_panicf(context, "Invalid number of arguments to #%s, wanted %zu, got %zu",
                 method->selector->name->data, method->argCount, array->size);
    }
    for (size_t i = 0; i < array->size; i++) {
      context->frame[i] = array->data[i];
    }
  }
  return context;
}

struct FooContext* foo_context_new_method_va(const struct FooMethod* method,
                                             struct FooContext* sender,
                                             const struct FooSelector* selector,
                                             struct Foo receiver,
                                             size_t nargs, va_list arguments) {
  // FOO_DEBUG("/foo_context_new_method_va");
  struct FooContext* context
    = foo_context_new_method_no_args(method, sender, receiver, method->argCount);
  if (selector != method->selector) {
    assert(&FOO_perform_with_ == method->selector);
    assert(method->argCount == 2);
    context->frame[0] = (struct Foo){ .class = &FooClass_Selector,
                                      .datum = { .ptr = (void*)selector }};
    struct FooArray* array = FooArray_alloc(nargs);
    for (size_t i = 0; i < nargs; i++) {
      array->data[i] = va_arg(arguments, struct Foo);
    }
    context->frame[1] = (struct Foo){ .class = &FooClass_Array,
                                      .datum = { .ptr = array } };
  } else {
    foo_check_method_argcount(sender, nargs, method);
    for (size_t i = 0; i < nargs; i++) {
      context->frame[i] = va_arg(arguments, struct Foo);
    }
  }
  return context;
}

bool foo_eq(struct Foo a, struct Foo b) {
  return a.class == b.class && a.datum.int64 == b.datum.int64;
}

void foo_print_backtrace(struct FooContext* context) {
  printf("Backtrace:\n");
  struct FooClass* home;
  struct FooClass* here;
  while (context && context->depth) {
    switch (context->type) {
    case METHOD_CONTEXT:
      home = context->method->class;
      here = context->receiver.class;
      printf("  %u: ", context->depth);
      struct FooSelector* selector = context->method->selector;
      printf("%s#%s", home->name->data, selector->name->data);
      if (here != home) {
        printf(" (%s)", here->name->data);
      }
      if (selector == &FOO_perform_with_ && context->size > 0) {
        struct Foo arg = context->frame[0];
        if (arg.class == &FooClass_Selector) {
          struct FooSelector* argSelector = PTR(FooSelector, arg.datum);
          printf(" #%s", argSelector->name->data);
        }
      }
      printf("\n");
      break;
    case CLOSURE_CONTEXT:
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
  assert(context);

  // Grab things from context before GC, makes debugging things like
  // accidentally collected contexts easier.
  const struct FooMethod* method = context->method;
  assert(method);
  struct FooClass* here = context->receiver.class;
  assert(here);
  struct FooClass* home = method->class;
  assert(home);
  uint32_t depth = context->depth;

  if (depth > 1000) {
    foo_panicf(context, "Stack blew up!");
  }
  foo_maybe_gc(context);

  jmp_buf ret;
  context->ret = &ret;
  int jmp = setjmp(ret);

  if (jmp) {
    FOO_DEBUG("/foo_activate(%u: %p) -> non-local return from %s#%s (%s)",
              depth, context,
              home->name->data,
              method->selector->name->data,
              here->name->data);
    assert(context->method == method);
    return context->return_value;
  } else {
    FooMethodFunction function = method->function;
    assert(function);
    FOO_DEBUG("/foo_activate(%u: %p) %s#%s (%s)",
              depth, context,
              home->name->data,
              method->selector->name->data,
              here->name->data);
    struct Foo res = function(method, context);
    FOO_DEBUG("/foo_activate(%u = %p) -> local return from %s#%s (%s)",
              depth, context,
              home->name->data,
              method->selector->name->data,
              here->name->data);
    assert(context->method == method);
    return res;
  }
}

struct Foo foo_send_array(struct FooContext* sender,
                          const struct FooSelector* selector,
                          struct Foo receiver,
                          struct Foo array) {
  // FOO_DEBUG("/foo_send_array(?, %s, ...)", selector->name->data);
  if (!receiver.class) {
    foo_panicf(sender, "Invalid receiver for #%s", selector->name->data);
  }
  const struct FooMethod* method
    = foo_class_find_method(sender, receiver.class, selector);
  struct FooContext* context
    = foo_context_new_method_array(method, sender, selector, receiver, array);
  return foo_activate(context);
}

struct Foo foo_send(struct FooContext* sender,
                    const struct FooSelector* selector,
                    struct Foo receiver,
                    size_t nargs, ...) {
  // FOO_DEBUG("/foo_send(?, %s, %s, ...)",
  //           selector->name->data, receiver.class->name->data);
  va_list arguments;
  va_start(arguments, nargs);
  if (!receiver.class) {
    foo_panicf(sender, "Invalid receiver for #%s", selector->name->data);
  }
  const struct FooMethod* method
    = foo_class_find_method(sender, receiver.class, selector);
  struct FooContext* context
    = foo_context_new_method_va(method, sender, selector, receiver, nargs, arguments);
  return foo_activate(context);
}


/**
 * Used as method function in methods implemented by objects. */
struct Foo foo_invoke_on(const struct FooMethod* method, struct FooContext* context) {
  struct FooArray* args = FooArray_alloc(method->argCount);
  for (size_t i = 0; i < args->size; i++) {
    args->data[i] = context->frame[i];
  }
  return foo_send(context, &FOO_invoke_on_, method->object,
                  2,
                  (struct Foo)
                  { .class = &FooClass_Array,
                    .datum = { .ptr = args } },
                  context->receiver);
}

struct Foo foo_method_doSelectors_(const struct FooMethod* method, struct FooContext* ctx) {
  (void)method;
  struct FooClass* vt = ctx->receiver.class;
  struct Foo block = ctx->frame[0];
  for (size_t i = 0; i < vt->size; i++) {
    foo_send(ctx, &FOO_value_, block, 1,
             (struct Foo){ .class = &FooClass_Selector,
                            .datum = { .ptr = vt->methods[i].selector } });
  }
  return ctx->receiver;
}

struct Foo foo_method_classOf(const struct FooMethod* method, struct FooContext* ctx) {
  (void)method;
  return (struct Foo){ .class = ctx->receiver.class->metaclass,
                       .datum = { .ptr = ctx->receiver.class } };
}

struct Foo foo_method_includes_(const struct FooMethod* method, struct FooContext* ctx) {
  (void)method;
  return foo_class_includes(ctx, PTR(FooClass, ctx->receiver.datum), ctx->frame[0]);
}

struct Foo foo_method_name(const struct FooMethod* method, struct FooContext* ctx) {
  (void)method;
  struct FooClass* class = PTR(FooClass, ctx->receiver.datum);
  return (struct Foo){ .class = &FooClass_String, .datum = { .ptr = class->name } };
}

struct Foo foo_closure_new(struct FooContext* context,
                           FooClosureFunction function,
                           size_t argCount,
                           size_t frameSize) {
  struct FooClosure* closure = foo_alloc(sizeof(struct FooClosure));
  closure->context = context;
  closure->function = function;
  closure->argCount = argCount;
  closure->frameSize = frameSize;
  return (struct Foo){ .class = &FooClass_Closure, .datum = { .ptr = closure } };
}

struct Foo FooGlobal_True =
  {
   .class = &FooClass_Boolean,
   .datum = { .boolean = 1 }
  };

struct Foo FooGlobal_False =
  {
   .class = &FooClass_Boolean,
   .datum = { .boolean = 0 }
  };

struct FooProcessTimes* FooProcessTimes_alloc() {
  return foo_alloc(sizeof(struct FooProcessTimes));
}

struct FooProcessTimes* FooProcessTimes_now() {
  struct FooProcessTimes* times = FooProcessTimes_alloc();
  system_get_process_times(times);
  return times;
}

struct FooProcessTimes* FooProcessTimes_new(double user, double system, double real) {
  struct FooProcessTimes* times = FooProcessTimes_alloc();
  times->user = user;
  times->system = system;
  times->real = real;
  return times;
}

struct FooArray* FooArray_alloc(size_t size) {
  struct FooArray* array = foo_alloc(sizeof(struct FooArray) + size*sizeof(struct Foo));
  array->header.allocation = HEAP;
  array->size = size;
  return array;
}

struct FooArray* FooInstance_alloc(size_t size) {
  struct FooArray* array = foo_alloc(sizeof(struct FooArray) + size*sizeof(struct Foo));
  array->header.allocation = HEAP;
  array->size = size;
  return array;
}

struct Foo foo_Array_new(size_t size) {
  struct FooArray* array = FooArray_alloc(size);
  for (size_t i = 0; i < size; ++i) {
    array->data[i] = FooGlobal_False;
  }
  return (struct Foo){ .class = &FooClass_Array, .datum = { .ptr = array } };
}

struct Foo foo_Array_alloc(size_t size) {
  struct FooArray* array = FooArray_alloc(size);
  return (struct Foo){ .class = &FooClass_Array, .datum = { .ptr = array } };
}

struct FooBytes* FooBytes_alloc(size_t len) {
  struct FooBytes* bytes = (struct FooBytes*)foo_alloc(sizeof(struct FooBytes) + len + 1);
  bytes->header.allocation = HEAP;
  bytes->size = len;
  return bytes;
}

struct FooBytes* FooBytes_from(const char* s) {
  size_t len = strlen(s);
  struct FooBytes* bytes = FooBytes_alloc(len);
  memcpy(bytes->data, s, len);
  return bytes;
}

struct Foo foo_String_new(size_t len, const char* s) {
  struct FooBytes* bytes = FooBytes_alloc(len);
  memcpy(bytes->data, s, len);
  return (struct Foo) { .class = &FooClass_String, .datum = { .ptr = bytes } };
}

struct Foo foo_String_new_from(const char* s) {
  return foo_String_new(strlen(s), s);
}

struct Foo foo_panic(struct FooContext* ctx, struct Foo message) {
  printf("PANIC: ");
  if (&FooClass_String == message.class) {
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

bool trace_gc = false;
size_t gc_trace_depth = 0;
#define ENTER_TRACE(...) if (trace_gc) { fprintf(stderr, "\n"); for(size_t i = 0; i < gc_trace_depth; i++) fprintf(stderr, "  "); fprintf(stderr, "%zu: ", gc_trace_depth); fprintf(stderr, __VA_ARGS__); gc_trace_depth++; }
#define EXIT_TRACE() if (trace_gc) { gc_trace_depth--; if (!gc_trace_depth) fprintf(stderr, "\n"); }

#if 0
#define DEBUG_GC(...) { fprintf(stderr, __VA_ARGS__); fflush(stderr); }
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
  ENTER_TRACE("mark_object");
  if (obj.class) {
    DEBUG_GC(" %p (%s)", obj.datum.ptr, obj.class->name->data);
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

void foo_mark_pointers(void* ptr) {
  struct FooPointerList* list = ptr;
  ENTER_TRACE("mark_pointers %p (size=%zu)", list, list->size);
  if (list->header.allocation == HEAP && foo_mark_live(list)) {
    for (size_t i = 0; i < list->size; i++) {
      list->mark(list->data[i]);
    }
  }
  EXIT_TRACE();
}

void foo_mark_class(void* ptr)
{
  struct FooClass* class = ptr;
  ENTER_TRACE("mark_class %p (%s)", class, class->name->data);
  assert(class);
  if (class->header.allocation == HEAP && foo_mark_live(class)) {
    foo_mark_bytes(class->name);
    foo_mark_class(class->metaclass);
    foo_mark_pointers(class->inherited);
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
  ENTER_TRACE("mark_context");
  if (ctx) {
    DEBUG_GC(" depth: %u, size: %zu", ctx->depth, ctx->size);
    if (ctx->type == METHOD_CONTEXT) {
      DEBUG_GC(" selector: %s", ctx->method->selector->name->data);
    }
  }
  if (ctx && foo_mark_live(ctx)) {
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
      fprintf(stderr, "** GC'd %zu bytes in %zu objects, %zu bytes in %zu objects remain.\n",
              freed_bytes, freed_count,
              allocation_bytes, allocation_count);
      fprintf(stderr, "** %zu bytes in %zu objects allocated since last gc.\n",
              allocation_bytes_since_gc, allocation_count_since_gc);
    }

    assert(live_count == allocation_count);
    assert(live_bytes == allocation_bytes);

    allocation_count_since_gc = 0;
    allocation_bytes_since_gc = 0;
  }
}

void foo_gc(struct FooContext* ctx) {
  FOO_DEBUG("/foo_gc begin");
  ENTER_TRACE("GC\n");

  // Mark everything dead
  struct FooAlloc* head = allocations;
  while (head) {
    head->mark = DEAD;
    head = head->next;
  }

  // Mark everything from ctx live
  if (ctx->vars) {
    ENTER_TRACE("vars");
    foo_mark_array(ctx->vars);
    EXIT_TRACE();
  }
  foo_mark_context(ctx);

  // Free dead things
  foo_sweep();

  EXIT_TRACE();
  FOO_DEBUG("/foo_gc end");
}

void foo_maybe_gc(struct FooContext* ctx) {
  if (allocation_bytes_since_gc > gc_threshold) {
    foo_gc(ctx);
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
  p->mark = LIVE;
  allocations = p;

  allocation_bytes_since_gc += bytes;
  allocation_bytes += bytes;
  allocation_count_since_gc += 1;
  allocation_count += 1;

  return p->data;
}

const size_t FooFile_READ      = 0b0001;
const size_t FooFile_WRITE     = 0b0010;
const size_t FooFile_APPEND    = 0b0100;
const size_t FooFile_TRUNCATE  = 0b1000;

const size_t FooFile_OPEN           = 0b01;
const size_t FooFile_CREATE         = 0b10;
const size_t FooFile_CREATE_OR_OPEN = 0b11;

struct Foo foo_File_new(struct FooBytes* path, size_t mode);
struct Foo foo_FileStream_new(struct FooContext* ctx, struct FooFile* file, size_t flags);

#include "generated_declarations.h"
#include "generated_constants.c"
#include "generated_builtins.c"
#include "generated_closures.c"
#include "generated_main.c"

struct Foo foo_File_new(struct FooBytes* pathname, size_t mode) {
  struct FooFile* file = foo_alloc(sizeof(struct FooFile));
  file->header.allocation = HEAP;
  file->pathname = pathname;
  file->mode = mode;
  return (struct Foo){ .class = &FooClass_File, .datum = { .ptr = file } };
}

struct Foo foo_FileStream_new(struct FooContext* ctx, struct FooFile* file, size_t flags) {
  // FIXME: GC should close stream!
  struct FooFileStream* stream = foo_alloc(sizeof(struct FooFileStream));
  stream->header.allocation = HEAP;
  stream->pathname = file->pathname;
  const char* mode = NULL;
  if (flags == FooFile_OPEN && file->mode == FooFile_READ) {
    mode = "rb";
  } else if (flags == FooFile_OPEN && file->mode == (FooFile_READ | FooFile_WRITE)) {
    mode = "r+b";
  } else if (flags == FooFile_CREATE_OR_OPEN && file->mode == (FooFile_TRUNCATE | FooFile_WRITE)) {
    mode = "wb";
  } else if (flags == FooFile_CREATE_OR_OPEN && file->mode == (FooFile_TRUNCATE | FooFile_READ | FooFile_WRITE)) {
    mode = "w+b";
  } else if (flags == FooFile_CREATE_OR_OPEN && file->mode == FooFile_APPEND) {
    mode = "ab";
  } else if (flags == FooFile_CREATE_OR_OPEN && file->mode == (FooFile_APPEND | FooFile_READ)) {
    mode = "a+b";
  } else {
    // Eg. open for append, don't create. FIXME: Need to implement on top of
    // open() instead, but windows compat via _sopen_s is more than one line for
    // that, so skipping for now.
    foo_panicf(ctx, "Unsupported file mode & flags: mode=%zu, flags=%zu!", file->mode, flags);
  }
  FOO_DEBUG("fopen(%s, %s)", (char*)file->pathname->data, mode);
  stream->ptr = fopen((char*)file->pathname->data, mode);
  if (!stream->ptr) {
    foo_panicf(ctx, "fdopen() failed!");
  }
  return (struct Foo){ .class = &FooClass_FileStream, .datum = { .ptr = stream } };
}
