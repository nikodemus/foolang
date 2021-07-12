#include <stdio.h>

#include "system.h"

int main() {
  double t0 = system_time_seconds();
  system_sleep(0.001);
  double t1 = system_time_seconds();
  system_sleep(0.01);
  double t2 = system_time_seconds();
  system_sleep(0.1);
  double t3 = system_time_seconds();
  system_sleep(1.0);
  double t4 = system_time_seconds();

  printf("0.000 %f\n", t0);
  printf("0.001 %f\n", t1-t0);
  printf("0.010 %f\n", t2-t1);
  printf("0.100 %f\n", t3-t2);
  printf("1.000 %f\n", t4-t3);
  return 0;
}
