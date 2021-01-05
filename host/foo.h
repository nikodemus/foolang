#ifndef __FOO_H_
#define __FOO_H_

struct FooProcessTimes {
  double user;
  double system;
  double real;
};

#define INSTANCE(class, value) \
  ((struct Foo){ .vtable = &FooInstanceVtable_ ## class, .datum = { .ptr = value } })

#define FLOAT(value) \
  ((struct Foo){ .vtable = &FooInstanceVtable_Float, .datum = { .float64 = value } })

#define INTEGER(value) \
  ((struct Foo){ .vtable = &FooInstanceVtable_Integer, .datum = { .int64 = value } })

#endif // __FOO_H_
