#include "error.h"

#include <stdio.h>
#include <stdlib.h>

char* runtime_type_error(char* sp, struct Actor* actor) {
  (void)sp;
  (void)actor;
  printf("Runtime type error!\n");
  exit(1);
  return 0;
}
