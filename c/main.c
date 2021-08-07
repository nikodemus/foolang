// FIXME: fopen() on Windows, should use fopen_s instead.
#define _CRT_SECURE_NO_WARNINGS 1

#include <ctype.h>
#include <errno.h>
#include <float.h>
#include <inttypes.h>
#include <math.h>
#include <setjmp.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
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
#include "mark-and-sweep.h"

size_t min_size(size_t a, size_t b) {
  if (a <= b) {
    return a;
  } else {
    return b;
  }
}

#define PTR(type, datum) \
  ((struct type*)datum.ptr)

void foo_unimplemented(const char* message) {
  printf("UNIMPLEMENTED: %s", message);
  fflush(stdout);
  fflush(stderr);
  _Exit(1);
}

void foo_abort(const char* message) {
  perror(message);
  fflush(stdout);
  fflush(stderr);
  _Exit(1);
}

#define FOO_CSTRING(literal) \
  ((struct FooBytes*) \
  &((struct { struct FooHeader header; size_t size; const char data[sizeof(literal)]; }) \
     { .header = { .allocation = STATIC }, .size = sizeof(literal)-1, .data = literal }))

bool foo_bytes_equal(const struct FooBytes* a, const struct FooBytes* b) {
  return a->size == b->size && !memcmp(a->data, b->data, a->size);
}

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

struct FooContext* foo_alloc_context(struct FooContext* sender, size_t size) {
  struct FooContext* ctx
    = foo_alloc_no_gc(sender, sizeof(struct FooContext) + size * sizeof(struct Foo));
  ctx->header.allocation = HEAP;
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
struct FooClassList FooClassInheritance_Class;
struct FooArray* FooArray_alloc(struct FooContext* sender, size_t size);
struct FooArray* FooArray_alloc_no_gc(struct FooContext* sender, size_t size);
struct Foo foo_Float_new(double f);
struct Foo foo_class_typecheck(struct FooContext* ctx, struct FooClass* class, struct Foo obj);
struct Foo FooGlobal_True;
struct Foo FooGlobal_False;
struct FooContext* foo_context_new_closure(struct FooContext* ctx);

struct FooContext* foo_context_new_main(struct FooArray* vars) {
  struct FooContext* context = foo_alloc_context(NULL, 0);
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
  struct FooContext* context = foo_alloc_context(sender, closure->frameSize);
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
  struct FooContext* context = foo_alloc_context(sender, closure->frameSize);
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

struct FooContext* foo_context_new_unwind(struct FooContext* sender, struct FooClosure* closure) {
  struct FooContext* context = foo_alloc_context(sender, closure->frameSize);
  context->type = UNWIND_CONTEXT;
  context->depth = sender->depth + 1;
  context->method = NULL;
  context->receiver = closure->context->receiver;
  context->sender = sender;
  context->outer_context = closure->context;
  context->vars = sender->vars;
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

struct FooClassList* FooClassList_alloc(struct FooContext* sender, size_t size) {
  struct FooClassList* list
    = foo_alloc(sender, sizeof(struct FooClassList) + size * sizeof(void*));
  list->header.allocation = HEAP;
  list->size = size;
  return list;
}

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

struct FooLayout* foo_Layout_new(struct FooContext* sender, size_t size) {
  // Empty layout can be shared, everything else needs to the unique: otherwise
  // having layout to one class would allow direct access to instance variables
  // of another class instance with shape.
  //
  // (The class layout is a special case, but user code should not be able
  // to access it.)
  if (size == 0)
    return &TheEmptyLayout;
  struct FooLayout* layout = foo_alloc(sender, sizeof(struct FooLayout));
  layout->header.allocation = HEAP;
  layout->mark = foo_mark_array;
  layout->size = size;
  return layout;
}

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
  struct FooArray* new = FooArray_alloc(ctx, theLayout->size);
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
  struct FooClassList* list = class->inherited;
  for (size_t i = 0; i < list->size; i++) {
    struct FooClass* ptr = list->data[i];
    if (want == ptr || foo_class_inherits(want, ptr)) {
      return true;
    }
  }
  return false;
}

struct Foo foo_class_typecheck(struct FooContext* sender,
                               struct FooClass* class,
                               struct Foo obj) {
  assert(class);
  if (!obj.class) {
    foo_panicf(sender, "Object has no class to check: %p, wanted %s",
               obj.datum.ptr, class->name->data);
  }
  if (class == obj.class || foo_class_inherits(class, obj.class)) {
    return obj;
  }
  foo_panicf(sender, "Type error! Wanted: %s, got: %s",
             class->name->data, obj.class->name->data);
}

struct FooContext* foo_context_new_method_dummy(const struct FooMethod* method,
                                                struct FooContext* sender,
                                                struct Foo receiver,
                                                struct Foo arg);

// For use in methods which don't allocate their own context:
// creates a mock context on failure.
struct Foo foo_class_typecheck_ctor_argument(struct FooContext* sender,
                                             struct FooClass* class,
                                             struct Foo obj,
                                             const struct FooMethod* method,
                                             size_t index) {
  assert(class);
  if (!obj.class) {
    foo_panicf(sender, "Object has no class to check: %p, wanted %s",
               obj.datum.ptr, class->name->data);
  }
  if (class == obj.class || foo_class_inherits(class, obj.class)) {
    return obj;
  }
  struct FooContext* dummy = foo_context_new_method_dummy(method, sender, sender->receiver, obj);
  foo_panicf(dummy, "Type error! Wanted: %s, got: %s",
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
  const struct FooClassList* list = class->inherited;
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
  assert(ctx);
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

// This is a dummy context for debugger and backtraces, containing only a single
// argument.
struct FooContext* foo_context_new_method_dummy(const struct FooMethod* method,
                                                struct FooContext* sender,
                                                struct Foo receiver,
                                                struct Foo arg) {
  struct FooContext* context = foo_alloc_context(sender, 1);
  context->type = METHOD_CONTEXT;
  context->depth = sender->depth + 1;
  context->method = method;
  context->sender = sender;
  context->receiver = receiver;
  context->outer_context = NULL;
  context->vars = sender->vars;
  context->frame[0] = arg;
  return context;
}

// This is a regular method context.
struct FooContext* foo_context_new_method_va(const struct FooMethod* method,
                                             struct FooContext* sender,
                                             const struct FooSelector* selector,
                                             struct Foo receiver,
                                             size_t frameSize,
                                             size_t nargs, va_list arguments) {
  if (selector != method->selector) {
    foo_panicf(sender, "Selector mismatch. Method %s#s activated through %s\n%zu arguments, frame size %zy",
               method->home->name->data,
               method->selector->name->data,
               selector->name->data,
               nargs, frameSize);
  }
  // FOO_DEBUG("/foo_context_new_method_va");
  struct FooContext* context = foo_alloc_context(sender, frameSize);
  context->type = METHOD_CONTEXT;
  context->depth = sender->depth + 1;
  context->method = method;
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
  return a.class == b.class && a.datum.int64 == b.datum.int64;
}

void foo_print_backtrace(struct FooContext* context) {
  printf("Backtrace:\n");
  struct FooClass* home;
  struct FooClass* here;
  while (context && context->depth) {
    switch (context->type) {
    case METHOD_CONTEXT:
      home = context->method->home;
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

struct Foo foo_send_perform_with(const struct FooMethod* method,
                                 struct FooContext* sender,
                                 struct Foo receiver, ...) {
  va_list arguments;
  va_start(arguments, receiver);
  struct Foo result = method->function(method, &FOO_perform_with_, sender, receiver, 2, arguments);
  va_end(arguments);
  return result;
}

struct Foo foo_send(struct FooContext* sender,
                    const struct FooSelector* selector,
                    struct Foo receiver,
                    size_t nargs, ...) {
  if (sender->depth > 2000) {
    foo_panicf(sender, "Stack blew up!");
  }
  va_list arguments;
  va_start(arguments, nargs);
  const struct FooMethod* method
    = foo_class_find_method(sender, receiver.class, selector);
  struct Foo result;
  if (method->selector != selector) {
    // This isn't the requested method, but rather #perform:with:
    struct FooArray* array = FooArray_alloc(sender, nargs);
    for (size_t i = 0; i < nargs; i++) {
      array->data[i] = va_arg(arguments, struct Foo);
    }
    result = foo_send_perform_with(method, sender, receiver,
                                   FOO_INSTANCE(Selector, selector), FOO_INSTANCE(Array, array));
  } else {
    result = method->function(method, selector, sender, receiver, nargs, arguments);
  }
  va_end(arguments);
  return result;
}

struct Foo foo_send_array(struct FooContext* sender,
                          const struct FooSelector* selector,
                          struct Foo receiver,
                          struct Foo argumentsArray) {
  assert(sender);
  if (sender->depth > 2000) {
    foo_panicf(sender, "Stack blew up!");
  }
  if (!receiver.class) {
    foo_panicf(sender, "Invalid receiver for #%s", selector->name->data);
  }
  struct FooArray* array = PTR(FooArray, argumentsArray.datum);
  switch (array->size) {
  case 0:
    return foo_send(sender, selector, receiver, 0);
  case 1:
    return foo_send(sender, selector, receiver, 1, array->data[0]);
  case 2:
    return foo_send(sender, selector, receiver, 2, array->data[0], array->data[1]);
  case 3:
    return foo_send(sender, selector, receiver, 3, array->data[0], array->data[1], array->data[2]);
  case 4:
    return foo_send(sender, selector, receiver, 4, array->data[0], array->data[1], array->data[2], array->data[3]);
  case 5:
    return foo_send(sender, selector, receiver, 5, array->data[0], array->data[1], array->data[2], array->data[3], array->data[4]);
  case 14:
    return foo_send(sender, selector, receiver, 14,
                    array->data[0], array->data[1], array->data[2], array->data[3], array->data[4],
                    array->data[5], array->data[6], array->data[7], array->data[8], array->data[9],
                    array->data[10], array->data[11], array->data[12], array->data[13]);
  default:
    foo_panicf(sender, "foo_send_array() not implemented for arrays of size %zu",
               array->size);
  }
}

/**
 * Used as method function in methods implemented by objects. */
struct Foo foo_invoke_on(const struct FooMethod* method,
                         const struct FooSelector* selector,
                         struct FooContext* sender,
                         struct Foo receiver,
                         size_t nargs, va_list arguments) {
  assert(method->selector == selector);
  struct FooArray* args = FooArray_alloc_no_gc(sender, nargs);
  for (size_t i = 0; i < nargs; i++) {
    args->data[i] = va_arg(arguments, struct Foo);
  }
  return foo_send(sender, &FOO_invoke_on_, method->object,
                  2,
                  FOO_INSTANCE(Array, args),
                  receiver);
}

struct Foo foo_method_classOf(const struct FooMethod* method,
                              const struct FooSelector* selector,
                              struct FooContext* sender,
                              struct Foo receiver,
                              size_t nargs, va_list arguments) {
  (void)method;
  (void)selector;
  (void)sender;
  (void)nargs;
  (void)arguments;
  return (struct Foo){ .class = receiver.class->metaclass,
                       .datum = { .ptr = receiver.class } };
}

struct Foo foo_method_includes_(const struct FooMethod* method,
                                const struct FooSelector* selector,
                                struct FooContext* sender,
                                struct Foo receiver,
                                size_t nargs, va_list arguments) {
  (void)method;
  (void)selector;
  (void)nargs;
  return foo_class_includes(sender, PTR(FooClass, receiver.datum), va_arg(arguments, struct Foo));
}

struct Foo foo_method_name(const struct FooMethod* method,
                           const struct FooSelector* selector,
                           struct FooContext* sender,
                           struct Foo receiver,
                           size_t nargs, va_list arguments) {
  (void)method;
  (void)selector;
  (void)sender;
  (void)nargs;
  (void)arguments;
  struct FooClass* class = PTR(FooClass, receiver.datum);
  return (struct Foo){ .class = &FooClass_String, .datum = { .ptr = class->name } };
}

struct Foo foo_closure_new(struct FooContext* sender,
                           FooClosureFunction function,
                           size_t argCount,
                           size_t frameSize) {
  struct FooClosure* closure = foo_alloc(sender, sizeof(struct FooClosure));
  closure->context = sender;
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

struct FooProcessTimes* FooProcessTimes_alloc(struct FooContext* sender) {
  return foo_alloc(sender, sizeof(struct FooProcessTimes));
}

struct FooProcessTimes* FooProcessTimes_now(struct FooContext* sender) {
  struct FooProcessTimes* times = FooProcessTimes_alloc(sender);
  system_get_process_times(times);
  return times;
}

struct FooProcessTimes* FooProcessTimes_new(struct FooContext* sender, double user, double system, double real) {
  struct FooProcessTimes* times = FooProcessTimes_alloc(sender);
  times->user = user;
  times->system = system;
  times->real = real;
  return times;
}

struct FooArray* FooArray_alloc(struct FooContext* sender, size_t size) {
  struct FooArray* array = foo_alloc(sender, sizeof(struct FooArray) + size*sizeof(struct Foo));
  array->header.allocation = HEAP;
  array->size = size;
  return array;
}

struct FooArray* FooArray_alloc_no_gc(struct FooContext* sender, size_t size) {
  struct FooArray* array = foo_alloc_no_gc(sender, sizeof(struct FooArray) + size*sizeof(struct Foo));
  array->header.allocation = HEAP;
  array->size = size;
  return array;
}

struct FooArray* FooInstance_alloc(struct FooContext* sender, size_t size) {
  struct FooArray* array = foo_alloc(sender, sizeof(struct FooArray) + size*sizeof(struct Foo));
  array->header.allocation = HEAP;
  array->size = size;
  return array;
}

struct Foo foo_Array_new(struct FooContext* sender, size_t size) {
  struct FooArray* array = FooArray_alloc(sender, size);
  for (size_t i = 0; i < size; ++i) {
    array->data[i] = FooGlobal_False;
  }
  return (struct Foo){ .class = &FooClass_Array, .datum = { .ptr = array } };
}

struct Foo foo_Array_alloc(struct FooContext* sender, size_t size) {
  struct FooArray* array = FooArray_alloc(sender, size);
  return (struct Foo){ .class = &FooClass_Array, .datum = { .ptr = array } };
}

struct FooBytes* FooBytes_alloc(struct FooContext* sender, size_t len) {
  struct FooBytes* bytes = (struct FooBytes*)foo_alloc(sender, sizeof(struct FooBytes) + len + 1);
  bytes->header.allocation = HEAP;
  bytes->size = len;
  return bytes;
}

struct FooBytes* FooBytes_from(struct FooContext* sender, const char* s) {
  size_t len = strlen(s);
  struct FooBytes* bytes = FooBytes_alloc(sender, len);
  memcpy(bytes->data, s, len);
  return bytes;
}

struct Foo foo_String_new(struct FooContext* sender, size_t len, const char* s) {
  struct FooBytes* bytes = FooBytes_alloc(sender, len);
  memcpy(bytes->data, s, len);
  return (struct Foo) { .class = &FooClass_String, .datum = { .ptr = bytes } };
}

struct Foo foo_String_new_from(struct FooContext* sender, const char* s) {
  return foo_String_new(sender, strlen(s), s);
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

const size_t FooFile_READ      = 0b0001;
const size_t FooFile_WRITE     = 0b0010;
const size_t FooFile_APPEND    = 0b0100;
const size_t FooFile_TRUNCATE  = 0b1000;

const size_t FooFile_OPEN           = 0b01;
const size_t FooFile_CREATE         = 0b10;
const size_t FooFile_CREATE_OR_OPEN = 0b11;

struct Foo foo_File_new(struct FooContext* sender, struct FooBytes* path, size_t mode);
struct Foo foo_FileStream_new(struct FooContext* ctx, struct FooFile* file, size_t flags);
struct Foo foo_Input_reopen(struct FooContext* sender, struct FooFile* file, size_t flags);

#include "generated_declarations.h"
#include "generated_constants.c"
#include "generated_builtins.c"
#include "generated_closures.c"
#include "generated_main.c"

struct Foo foo_File_new(struct FooContext* sender, struct FooBytes* pathname, size_t mode) {
  struct FooFile* file = foo_alloc(sender, sizeof(struct FooFile));
  file->header.allocation = HEAP;
  file->pathname = pathname;
  file->mode = mode;
  return (struct Foo){ .class = &FooClass_File, .datum = { .ptr = file } };
}

struct Foo foo_FileStream_new(struct FooContext* ctx, struct FooFile* file, size_t flags) {
  // FIXME: GC should close stream!
  struct FooFileStream* stream = foo_alloc(ctx, sizeof(struct FooFileStream));
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
    foo_panicf(ctx, "fopen(\"%s\", \"%s\") failed: %s",
               (char*)file->pathname->data, mode, strerror(errno));
  }
  return (struct Foo){ .class = &FooClass_FileStream, .datum = { .ptr = stream } };
}

struct Foo foo_Input_reopen(struct FooContext* ctx, struct FooFile* file, size_t flags) {
  const char* mode = NULL;
  // FIXME: These don't all make sensor for stdin...
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
  FOO_DEBUG("freopen(%s, %s, stdin)", (char*)file->pathname->data, mode);
  if (!freopen((char*)file->pathname->data, mode, stdin)) {
    foo_panicf(ctx, "freopen() failed!");
  }
  return FOO_INSTANCE(Input, stdin);
}
