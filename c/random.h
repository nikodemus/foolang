#ifndef __RANDOM_H_
#define __RANDOM_H_

#include <stdint.h>

struct FooRandom;
struct FooContext;

uint64_t foo_random_next(struct FooRandom* random);
void foo_random_jump(struct FooRandom* random);
void foo_random_long_jump(struct FooRandom* random);
uint64_t foo_random_fast(uint64_t* state);
void foo_random_init(struct FooRandom* random, uint64_t seed);

struct FooRandom* FooRandom_new(struct FooContext* sender, uint64_t seed);

#endif // __RANDOM_H_
