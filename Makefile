.PHONY: all

CC := clang
CPPFLAGS := -I.
CFLAGS := -g -Wall -Wextra -fsanitize=address -fsanitize=undefined

SRCS=$(wildcard runtime/*.c)
OBJS=$(SRCS:.c=.o)

DEPFLAGS = -MT $@ -MMD -MP -MF $*.d

COMPILE.c = $(CC) $(DEPFLAGS) $(CFLAGS) $(CPPFLAGS) -c

%.o: %.c %.d Makefile
	@echo -n .
	@$(COMPILE.c) $(OUTPUT_OPTION) $<

DEPFILES := $(SRCS:.c=.d)
$(DEPFILES):
include $(wildcard $(DEPFILES))

build/test-runtime: $(OBJS)
	@clang -o build/test-runtime $(OBJS) $(CFLAGS)

test: build/test-runtime
	@build/test-runtime
