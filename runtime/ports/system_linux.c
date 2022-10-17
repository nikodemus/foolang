#include <stdlib.h>
#include <sys/sysinfo.h>

size_t system_number_of_cpu_cores(void) {
  int n = get_nprocs();
  if (n < 0) {
    return 0;
  } else {
    return n;
  }
}
