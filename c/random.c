#include "foo.h"
#include "mark-and-sweep.h"

#include <math.h>
#include <stdint.h>

// This is https://prng.di.unimi.it/xoshiro256plusplus.c, downloaded
// 2021-08-10.
//
// Modifications: static state variable `s[4]` converted to FooRandom.state,
// names prefix with foo_random_, added foo_random_init as a wrapper for
// the splitmix64 generator.

/*  Written in 2019 by David Blackman and Sebastiano Vigna (vigna@acm.org)

To the extent possible under law, the author has dedicated all copyright
and related and neighboring rights to this software to the public domain
worldwide. This software is distributed without any warranty.

See <http://creativecommons.org/publicdomain/zero/1.0/>. */

/* This is xoshiro256++ 1.0, one of our all-purpose, rock-solid generators.
   It has excellent (sub-ns) speed, a state (256 bits) that is large
   enough for any parallel application, and it passes all tests we are
   aware of.

   For generating just floating-point numbers, xoshiro256+ is even faster.

   The state must be seeded so that it is not everywhere zero. If you have
   a 64-bit seed, we suggest to seed a splitmix64 generator and use its
   output to fill s. */

static inline uint64_t rotl(const uint64_t x, int k) {
  return (x << k) | (x >> (64 - k));
}

uint64_t foo_random_next(struct FooRandom* random) {
  const uint64_t result = rotl(random->state[0] + random->state[3], 23) + random->state[0];
  const uint64_t t = random->state[1] << 17;

  random->state[2] ^= random->state[0];
  random->state[3] ^= random->state[1];
  random->state[1] ^= random->state[2];
  random->state[0] ^= random->state[3];

  random->state[2] ^= t;

  random->state[3] = rotl(random->state[3], 45);

  return result;
}

/* This is the jump function for the generator. It is equivalent
   to 2^128 calls to next(); it can be used to generate 2^128
   non-overlapping subsequences for parallel computations. */
void foo_random_jump(struct FooRandom* random) {
  static const uint64_t JUMP[] = { 0x180ec6d33cfd0aba, 0xd5a61266f0c9392c, 0xa9582618e03fc9aa, 0x39abdc4529b1661c };

  uint64_t s0 = 0;
  uint64_t s1 = 0;
  uint64_t s2 = 0;
  uint64_t s3 = 0;
  for(int i = 0; i < sizeof JUMP / sizeof *JUMP; i++)
    for(int b = 0; b < 64; b++) {
      if (JUMP[i] & UINT64_C(1) << b) {
        s0 ^= random->state[0];
        s1 ^= random->state[1];
        s2 ^= random->state[2];
        s3 ^= random->state[3];
      }
      foo_random_next(random);
    }

  random->state[0] = s0;
  random->state[1] = s1;
  random->state[2] = s2;
  random->state[3] = s3;
}

/* This is the long-jump function for the generator. It is equivalent to
   2^192 calls to next(); it can be used to generate 2^64 starting points,
   from each of which jump() will generate 2^64 non-overlapping
   subsequences for parallel distributed computations. */
void foo_random_long_jump(struct FooRandom* random) {
  static const uint64_t LONG_JUMP[] = { 0x76e15d3efefdcbbf, 0xc5004e441c522fb3, 0x77710069854ee241, 0x39109bb02acbe635 };

  uint64_t s0 = 0;
  uint64_t s1 = 0;
  uint64_t s2 = 0;
  uint64_t s3 = 0;
  for(int i = 0; i < sizeof LONG_JUMP / sizeof *LONG_JUMP; i++)
    for(int b = 0; b < 64; b++) {
      if (LONG_JUMP[i] & UINT64_C(1) << b) {
        s0 ^= random->state[0];
        s1 ^= random->state[1];
        s2 ^= random->state[2];
        s3 ^= random->state[3];
      }
      foo_random_next(random);
    }

  random->state[0] = s0;
  random->state[1] = s1;
  random->state[2] = s2;
  random->state[3] = s3;
}

// This is the https://prng.di.unimi.it/splitmix64.c, downloaded
// 2021-08-10.
//
// Modifications: renamed next() to foo_random_fast, converted
// to taking a pointer to state instead of using a static x.

/*  Written in 2015 by Sebastiano Vigna (vigna@acm.org)

To the extent possible under law, the author has dedicated all copyright
and related and neighboring rights to this software to the public domain
worldwide. This software is distributed without any warranty.

See <http://creativecommons.org/publicdomain/zero/1.0/>. */

/* This is a fixed-increment version of Java 8's SplittableRandom generator
   See http://dx.doi.org/10.1145/2714064.2660195 and
   http://docs.oracle.com/javase/8/docs/api/java/util/SplittableRandom.html

   It is a very fast generator passing BigCrush, and it can be useful if
   for some reason you absolutely want 64 bits of state. */
uint64_t foo_random_fast(uint64_t* state) {
  uint64_t z = (*state += 0x9e3779b97f4a7c15);
  z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9;
  z = (z ^ (z >> 27)) * 0x94d049bb133111eb;
  return z ^ (z >> 31);
}

// This is code written on top of the above functionality

void foo_random_init(struct FooRandom* random, uint64_t seed) {
  random->state[0] = foo_random_fast(&seed);
  random->state[1] = foo_random_fast(&seed);
  random->state[2] = foo_random_fast(&seed);
  random->state[3] = foo_random_fast(&seed);
}

struct FooRandom* FooRandom_new(struct FooContext* sender, uint64_t seed) {
  struct FooRandom* random = foo_alloc(sender, sizeof(struct FooRandom));
  foo_random_init(random, seed);
  return random;
}
