#ifndef __EXT_H_
#define __EXT_H_

char * dtoa(double dd, int mode, int ndigits, int *decpt, int *sign, char **rve);

#define XXH_INLINE_ALL
#include "ext/xxhash.h"

#endif // __EXT_H_
