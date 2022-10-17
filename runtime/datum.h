#ifndef __DATUM_H_
#define __DATUM_H_

#include <stdint.h>

typedef int64_t datum_t;

#define READ_DATUM(p, offs) \
  (((datum_t*)p)[offs])

#define WRITE_DATUM(p, offs, datum) \
  (((datum_t*)p)[offs] = (datum))

#define PUSH_DATUM(p, datum) \
  { p += sizeof(datum_t); *((datum_t*)p) = (datum_t)datum; }

#endif // __DATUM_H_
