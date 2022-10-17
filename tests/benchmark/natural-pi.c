#include <inttypes.h>
#include <setjmp.h>
#include <stdio.h>
#include <stdlib.h>

#undef NDEBUG
#include <assert.h>

#include "pi_config.h"

double random_double() {
  return (double)rand()/(double)RAND_MAX;
}

typedef double (*PiFun)(void);

double dontpi(void) { return 0.0; };

double dopi(void) {
  int64_t inside = 0;
  int64_t n = N_ITERATIONS;
  for (int64_t i = 0; i < n; i++) {
    double x = random_double();
    double y = random_double();
    x = x*x;
    y = y*y;
    if (x + y < 1.0) {
      inside++;
    }
  }
  double ratio = 4.0 * (double)inside / N_ITERATIONS;
  return ratio;
}

PiFun mypi = dontpi;

int main(int argc, char** argv) {
  (void)argv;
  if (argc == 1) {
    mypi = dopi;
  }
  double gotpu = mypi();
  printf("    pi = %.2f\n", gotpu);
  return 0;
}
