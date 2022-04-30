#ifndef __THREAD_H_
#define __THREAD_H_

// When creating threads we pass in a specification, so that
// platform specific code can take care of invoking it the right
// way.

typedef void (*ThreadFunction)(void*);

struct ThreadInfo {
  ThreadFunction function;
  void* parameter;
};

struct ThreadInfo* make_ThreadInfo(ThreadFunction, void*);
void free_ThreadInfo(struct ThreadInfo*);

#endif // __THREAD_H_
