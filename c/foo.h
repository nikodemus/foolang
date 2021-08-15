#ifndef __FOO_H_
#define __FOO_H_

#include <stdarg.h>
#include <inttypes.h>
#include <stdio.h>
#include <setjmp.h>

#include "random.h"
#include "ext.h"

#if 0
# define FOO_DEBUG(...) { fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); fflush(stderr); }
#else
# define FOO_DEBUG(...)
#endif

#define FOO_XXX(...) { fprintf(stderr, "XXX: "); fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); fflush(stderr); }

#define PTR(type, datum) \
  ((struct type*)datum.ptr)

struct FooContext;
struct FooClass;

inline uint64_t foo_hash(struct FooClass* class, const void* data, size_t size) {
  return XXH3_64bits_withSeed(data, size, (uintptr_t)class);
}

inline uint64_t foo_identity_hash(const void* ptr) {
  return XXH3_64bits(&ptr, sizeof(ptr));
}

inline uint64_t foo_hashmix(uint64_t a, uint64_t b) {
  a ^= b + 0x9e3779b9 + (a << 6) + (b >> 2);
  return a;
}

struct Foo foo_panic(struct FooContext* ctx, struct Foo message) __attribute__((noreturn));
struct Foo foo_panicf(struct FooContext* ctx, const char* fmt, ...) __attribute__((noreturn));
void foo_unimplemented(const char* message) __attribute__ ((noreturn));
void foo_abort(const char* message) __attribute__ ((noreturn));

// XXX: Doesn't belong in here.
typedef void (*FooMarkFunction)(void* ptr);

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
  HEAP,
  STACK
};

struct FooHeader {
  enum FooAllocType allocation;
};

struct FooArray {
  struct FooHeader header;
  size_t size;
  struct Foo data[];
};

struct FooBytes {
  struct FooHeader header;
  size_t size;
  uint8_t data[];
};

/** Simple intrusive list for interning. O(N), but fine to start with.
 */
struct FooSelector {
  struct FooSelector* next;
  struct FooBytes* name;
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

// FIXME: vars doesn't really belong in context, but easier right now.
#define FOOCONTEXT_IMPL(prefix, frameSize)            \
  prefix {                                            \
    struct FooHeader header;                          \
    enum FooContextType type;                         \
    uint32_t depth;                                   \
    uint64_t heap;                                    \
    const struct FooMethod* method;                   \
    struct Foo receiver;                              \
    struct FooContext* sender;                        \
    struct FooContext* outer_context;                 \
    struct FooArray* vars;                            \
    struct FooCleanup* cleanup;                       \
    struct Foo return_value;                          \
    jmp_buf* ret;                                     \
    size_t size;                                      \
    struct Foo frame frameSize;                       \
}
FOOCONTEXT_IMPL(struct FooContext, []);
#define FOOCONTEXT_OF_SIZE(frameSize) (struct FooContext*)&(FOOCONTEXT_IMPL(struct,[frameSize]))

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

typedef struct Foo (*FooMethodFunction)
    (const struct FooMethod*,
     const struct FooSelector*,
     struct FooContext*,
     struct Foo,
     size_t,
     va_list);
typedef struct Foo (*FooClosureFunction)(struct FooContext*);

struct FooMethod {
  struct FooClass* home;
  struct FooSelector* selector;
  // Native method functions directly implement the method
  // Object method functions send #invoke:inContext: to the object
  FooMethodFunction function;
  struct Foo object;
};

struct FooLayout {
  struct FooHeader header;
  FooMarkFunction mark;
  size_t size;
};

struct FooClass {
  struct FooHeader header;
  struct FooBytes* name;
  struct FooClass* metaclass;
  struct FooClassList* inherited;
  struct FooLayout* layout;
  FooMarkFunction mark;
  size_t size;
  struct FooMethod methods[];
};

struct FooClassList {
  struct FooHeader header;
  size_t size;
  struct FooClass* data[];
};

struct FooClosure {
  struct FooContext* context;
  size_t argCount;
  size_t frameSize;
  FooClosureFunction function;
};

struct FooProcessTimes {
  double user;
  double system;
  double real;
};

void foo_finally(struct FooContext* sender, struct FooCleanup* cleanup);
void foo_unbind(struct FooContext* sender, struct FooCleanup* cleanup);

extern struct FooLayout TheEmptyLayout;
extern struct FooLayout TheClassLayout;
extern struct FooClass FooClass_Float;
extern struct FooClass FooClass_Boolean;
extern struct FooClass FooClass_Integer;
extern struct FooClass FooClass_Character;

#define FOO_INSTANCE(classname, value) \
  ((struct Foo){ .class = &FooClass_ ## classname, .datum = { .ptr = (void*)(value) } })

#define FOO_FLOAT(value) \
  ((struct Foo){ .class = &FooClass_Float, .datum = { .float64 = (double)(value) } })

#define FOO_INTEGER(value) \
  ((struct Foo){ .class = &FooClass_Integer, .datum = { .int64 = (int64_t)(value) } })

#define FOO_BOOLEAN(value) \
  ((struct Foo){ .class = &FooClass_Boolean, .datum = { .boolean = (bool)(value) } })

#define FOO_CHARACTER(value) \
  ((struct Foo){ .class = &FooClass_Character, .datum = { .int64 = (int64_t)(value) } })

#endif // __FOO_H_
