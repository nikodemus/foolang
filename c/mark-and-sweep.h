#ifndef __MARK_AND_SWEEP_H_
#define __MARK_AND_SWEEP_H_

#include <stdbool.h>
#include <stddef.h>

void foo_mark_array(void* ptr);
void foo_mark_bytes(void* ptr);
void foo_mark_class(void* ptr);
void foo_mark_closure(void* ptr);
void foo_mark_file(void* ptr);
void foo_mark_filestream(void* ptr);
void foo_mark_instance(void* ptr);
void foo_mark_layout(void* ptr);
void foo_mark_none(void* ptr);
void foo_mark_oops(void* ptr);
void foo_mark_ptr(void* ptr);

struct FooContext;

void* foo_alloc(struct FooContext*, size_t);
void foo_gc(struct FooContext*);

extern bool trace_gc;
extern bool gc_verbose;

#endif // __MARK-AND-SWEEP_H_
