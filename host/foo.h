#ifndef __FOO_H_
#define __FOO_H_

struct FooProcessTimes {
  double user;
  double system;
  double real;
};

#define INSTANCE(classname, value) \
  ((struct Foo){ .class = &FooClass_ ## classname, .datum = { .ptr = value } })

#define FLOAT(value) \
  ((struct Foo){ .class = &FooClass_Float, .datum = { .float64 = value } })

#define FOO_INTEGER(value) \
  ((struct Foo){ .class = &FooClass_Integer, .datum = { .int64 = (int64_t)(value) } })

#define FOO_BOOLEAN(value) \
  ((struct Foo){ .class = &FooClass_Boolean, .datum = { .boolean = (bool)(value) } })

#define FOO_CHARACTER(value) \
  ((struct Foo){ .class = &FooClass_Character, .datum = { .int64 = (int64_t)(value) } })

#endif // __FOO_H_
