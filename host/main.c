// FIXME: fopen() on Windows, should use fopen_s instead.
#define _CRT_SECURE_NO_WARNINGS 1

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

/** Out of line allocation of data for compatibility with C string literals.
 *
 *  FIXME: I'd really like to replace these with String.
 *  The big issue is that these cannot be passed out from selectors without
 *  copying. At least a class is needed...
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
  bool gc;
  size_t size;
  struct Foo data[];
};

struct FooBytes {
  bool gc;
  size_t size;
  uint8_t data[];
};

// FIXME: Don't like defining this in C.
struct FooFile {
  bool gc;
  struct FooBytes* pathname;
  size_t mode;
};

// FIXME: Don't like defining this in C.
struct FooFileStream {
  bool gc;
  struct FooBytes* pathname;
  FILE* ptr;
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

typedef struct Foo (*FooMethodFunction)(struct FooContext*);
typedef struct Foo (*FooClosureFunction)(struct FooContext*);

struct Foo foo_lexical_ref(struct FooContext* context, size_t index, size_t frameOffset) {
  struct FooContext* context0 = context;
  size_t frameOffset0 = frameOffset;
  FOO_DEBUG("/lexical_ref(index=%zu, frame=%zu)", index, frameOffset);
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
  // The class in which this method originates.
  struct FooClass* class;
  struct FooSelector* selector;
  size_t argCount;
  size_t frameSize;
  FooMethodFunction function;
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
struct FooArray* FooArray_alloc(size_t size);
struct Foo foo_Float_new(double f);
struct Foo foo_Integer_new(int64_t n);
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

struct FooClassList {
  size_t size;
  struct FooClass* data[];
};

typedef void (*FooMarkFunction)(void* ptr);

struct FooClass {
  struct FooCString* name;
  struct FooClass* metaclass;
  struct FooClassList* inherited;
  FooMarkFunction mark;
  size_t size;
  struct FooMethod methods[];
};

struct Foo foo_class_typecheck(struct FooContext* ctx,
                                struct FooClass* class,
                                struct Foo obj) {
  assert(class);
  assert(obj.class);
  if (class == obj.class)
    return obj;
  struct FooClassList* list = obj.class->inherited;
  for (size_t i = 0; i < list->size; i++)
    if (class == list->data[i]) {
      return obj;
  }
  foo_panicf(ctx, "Type error! Wanted: %s, got: %s",
             class->name->data, obj.class->name->data);
}

struct Foo foo_Boolean_new(bool t);

struct Foo foo_class_includes(struct FooContext* ctx,
                               struct FooClass* class,
                               struct Foo obj) {
  assert(class);
  assert(obj.class);
  if (class == obj.class)
    return foo_Boolean_new(true);
  struct FooClassList* list = obj.class->inherited;
  for (size_t i = 0; i < list->size; i++)
    if (class == list->data[i]) {
      return foo_Boolean_new(true);
  }
  return foo_Boolean_new(false);
}

const struct FooMethod* foo_class_find_method(struct FooContext* ctx,
                                               const struct FooClass* class,
                                               const struct FooSelector* selector) {
  assert(class);
  FOO_DEBUG("/foo_class_find_method(%s#%s)", class->name->data, selector->name->data);
  const struct FooMethod* fallback = NULL;
  for (size_t i = 0; i < class->size; ++i) {
    const struct FooMethod* method = &class->methods[i];
    if (method->selector == selector) {
      return method;
    } else if (method->selector == &FOO_perform_with_) {
      fallback = method;
    }
  }
  if (fallback) {
    FOO_DEBUG(" => fallback: %s", fallback->selector->name->data);
    return fallback;
  }
  /*
  for (size_t i = 0; i < class->size; ++i) {
    const struct FooMethod* method = &class->methods[i];
    printf("- %s\n", method->selector->name->data);
  }
  */
  foo_panicf(ctx, "%s does not understand: #%s", class->name->data, selector->name->data);
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
    assert(nargs == method->argCount);
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
  if (context->depth > 200) {
    foo_panicf(context, "Stack blew up!");
  }
  foo_maybe_gc(context);
  jmp_buf ret;
  context->ret = &ret;
  int jmp = setjmp(ret);
  if (jmp) {
    FOO_DEBUG("/foo_send -> non-local return from %s", context->method->selector->name->data);
    return context->return_value;
  } else {
    struct Foo res = context->method->function((struct FooContext*)context);
    FOO_DEBUG("/foo_send -> local return from %s", context->method->selector->name->data);
    return res;
  }
}

struct Foo foo_send_array(struct FooContext* sender,
                          const struct FooSelector* selector,
                          struct Foo receiver,
                          struct Foo array) {
  FOO_DEBUG("/foo_send_array(?, %s, ...)", selector->name->data);
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
  FOO_DEBUG("/foo_send(?, %s, ...)", selector->name->data);
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

struct Foo foo_method_doSelectors_(struct FooContext* context) {
  struct FooClass* vt = context->receiver.class;
  struct Foo block = context->frame[0];
  for (size_t i = 0; i < vt->size; i++) {
    foo_send(context, &FOO_value_, block, 1,
             (struct Foo){ .class = &FooClass_Selector,
                           .datum = { .ptr = vt->methods[i].selector } });
  }
  return context->receiver;
}

struct Foo foo_method_classOf(struct FooContext* ctx) {
  return (struct Foo){ .class = ctx->receiver.class->metaclass,
                        .datum = { .ptr = ctx->receiver.class } };
}

struct Foo foo_method_includes_(struct FooContext* ctx) {
  return foo_class_includes(ctx, PTR(FooClass, ctx->receiver.datum), ctx->frame[0]);
}

struct Foo foo_method_name(struct FooContext* ctx) {
  // FIXME: replace FooCStrings with just strings, so we can return name directly.
  struct FooClass* class = PTR(FooClass, ctx->receiver.datum);
  struct FooCString* name = class->name;
  return foo_String_new(name->size, name->data);
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
  array->gc = true;
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

struct Foo foo_Boolean_new(bool t) {
  return (struct Foo){ .class = &FooClass_Boolean, .datum = { .boolean = t } };
}

struct Foo foo_Character_new(int64_t n) {
  assert(n >= 0);
  return (struct Foo){ .class = &FooClass_Character, .datum = { .int64 = n } };
}

struct Foo foo_Integer_new(int64_t n) {
  return (struct Foo){ .class = &FooClass_Integer, .datum = { .int64 = n } };
}

struct Foo foo_Float_new(double f) {
  return (struct Foo){ .class = &FooClass_Float, .datum = { .float64 = f } };
}


struct FooBytes* FooBytes_alloc(size_t len) {
  struct FooBytes* bytes = (struct FooBytes*)foo_alloc(sizeof(struct FooBytes) + len + 1);
  bytes->gc = true;
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

#if 0
size_t gc_trace_depth = 0;
#define DEBUG_GC(...) { printf(__VA_ARGS__); fflush(stdout); }
#define ENTER_TRACE(...) { printf("\n"); for(size_t i = 0; i < gc_trace_depth; i++) printf("  "); printf("%zu: ", gc_trace_depth); printf(__VA_ARGS__); gc_trace_depth++; }
#define EXIT_TRACE() { gc_trace_depth--; if (!gc_trace_depth) printf("\n"); }
#else
#define DEBUG_GC(...)
#define ENTER_TRACE(...)
#define EXIT_TRACE()
#endif

enum FooMark {
  RED = 0,
  BLUE = 1,
};

static enum FooMark current_live_mark = RED;

void foo_flip_mark() {
  current_live_mark = !current_live_mark;
}

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
  bool new_mark = alloc->mark != current_live_mark;
  DEBUG_GC(" mark=%d, live=%d", alloc->mark, current_live_mark);
  alloc->mark = current_live_mark;
  EXIT_TRACE();
  return new_mark;
}

void foo_mark_ptr(void* ptr) {
  ENTER_TRACE("mark_ptr");
  foo_mark_live(ptr);
  EXIT_TRACE();
}

// FIXME: Move bytes->gc, etc into allocation header, allocating
// global objects as (struct Allocation){ .gc = false, ... }.
void foo_mark_bytes(void* ptr) {
  ENTER_TRACE("mark_bytes");
  struct FooBytes* bytes = ptr;
  if (bytes->gc) {
    foo_mark_live(ptr);
  }
  EXIT_TRACE();
}

void foo_mark_file(void* ptr) {
  ENTER_TRACE("mark_bytes");
  struct FooFile* file = ptr;
  if (file->gc) {
    foo_mark_live(file);
    foo_mark_bytes(file->pathname);
  }
  EXIT_TRACE();
}

void foo_mark_filestream(void* ptr) {
  ENTER_TRACE("mark_bytes");
  struct FooFileStream* stream = ptr;
  if (stream->gc) {
    foo_mark_live(stream);
    foo_mark_bytes(stream->pathname);
  }
  EXIT_TRACE();
}

void foo_mark_context(struct FooContext* ctx);

void foo_mark_object(struct Foo obj) {
  ENTER_TRACE("mark_object");
  if (obj.class) {
    DEBUG_GC(" %p (%s)", obj.datum.ptr, obj.class->name->data);
    obj.class->mark(obj.datum.ptr);
  }
  EXIT_TRACE();
}

void foo_mark_none(void* ptr) {
  (void)ptr;
}

void foo_mark_array(void* ptr) {
  ENTER_TRACE("mark_array");
  struct FooArray* array = ptr;
  if (array->gc && foo_mark_live(array)) {
    for (size_t i = 0; i < array->size; i++) {
      foo_mark_object(array->data[i]);
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
    ENTER_TRACE("--GC--\n");
    foo_flip_mark();
    if (ctx->vars) {
      foo_mark_array(ctx->vars);
    }
    foo_mark_context(ctx);
    foo_sweep();
    EXIT_TRACE();
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
  file->gc = true;
  file->pathname = pathname;
  file->mode = mode;
  return (struct Foo){ .class = &FooClass_File, .datum = { .ptr = file } };
}

struct Foo foo_FileStream_new(struct FooContext* ctx, struct FooFile* file, size_t flags) {
  // FIXME: GC should close stream!
  struct FooFileStream* stream = foo_alloc(sizeof(struct FooFileStream));
  stream->gc = true;
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
  stream->ptr = fopen((char*)file->pathname->data, mode);
  if (!stream->ptr) {
    foo_panicf(ctx, "fdopen() failed!");
  }
  return (struct Foo){ .class = &FooClass_FileStream, .datum = { .ptr = stream } };
}
