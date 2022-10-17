#include "system_posix.h"

#include <stdlib.h>

#undef NDEBUG
#include <assert.h>

#if defined(__linux__)
#include "system_linux.c"
#else
#include "system_generic.c"
#endif

SystemLock_t make_SystemLock(void) {
  pthread_mutex_t* mutex = malloc(sizeof(pthread_mutex_t));
  assert(mutex);
  assert(!pthread_mutex_init(mutex, NULL));
  return mutex;
}

void free_SystemLock(pthread_mutex_t* mutex) {
  assert(!pthread_mutex_destroy(mutex));
  free(mutex);
}

void system_lock(pthread_mutex_t* mutex) {
  assert(!pthread_mutex_lock(mutex));
}

void system_unlock(pthread_mutex_t* mutex) {
  assert(!pthread_mutex_unlock(mutex));
}

struct ThreadInfo {
    ThreadFunction function;
    void* parameter;
};

void* run_thread(void* data) {
  struct ThreadInfo* info = data;
  ThreadFunction function = info->function;
  void* parameter = info->parameter;
  free(info);
  function(parameter);
  return NULL;
}

SystemThread_t make_SystemThread(ThreadFunction function, void* parameter) {
  struct ThreadInfo* info = malloc(sizeof(struct ThreadInfo));
  info->function = function;
  info->parameter = parameter;
  pthread_t* thread = malloc(sizeof(pthread_t));
  assert(!pthread_create(thread, NULL, run_thread, info));
  return thread;
}
    
bool system_join_thread(pthread_t* thread) {
  void* res;
  bool ok = !pthread_join(*thread, &res);
  if (ok) {
    free(thread);
    return true;
  } else {
    return false;
  }
}

void system_sleep_ms(size_t ms) {
  struct timespec ts = { .tv_sec = ms / 1000,
			 .tv_nsec = (ms % 1000) * 1000000 };
  nanosleep(&ts, NULL);
}
