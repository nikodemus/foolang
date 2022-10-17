#include <stdlib.h>
#include <sys/types.h>
#include <sys/sysctl.h>

#undef NDEBUG
#include <assert.h>

size_t system_number_of_cpu_cores(void) {
  int n;
  size_t n_size = sizeof(n);
  assert(!sysctlbyname("hw.logicalcpu", &n, &n_size, NULL, 0));
  if (n < 0)
    return 0;
  else
    return (size_t)n;
}
