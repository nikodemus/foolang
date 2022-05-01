.PHONY: all
all:
	@echo "targets:"
	@echo " - test (unit tests for new runtime)"

CC := clang
CPPFLAGS := -I.
CFLAGS := -g -Wall -Wextra -fsanitize=address -fsanitize=undefined

ifeq ($(OS), Windows_NT)
	EXE=.exe
else
	EXE=
endif

SRCS=$(wildcard runtime/*.c)
OBJS=$(SRCS:%.c=build/%.o)

DEPFLAGS = -MT $@ -MMD -MP -MF $*.d

COMPILE.c = $(CC) $(DEPFLAGS) $(CFLAGS) $(CPPFLAGS) -c

build/%.o : %.c %.d Makefile
	@echo -n .
	@mkdir -p $(@D)
	@$(COMPILE.c) $(OUTPUT_OPTION) $<

DEPFILES := $(SRCS:%.c=build/%.d)
$(DEPFILES):

include $(wildcard $(DEPFILES))

build/test-runtime$(EXE): $(OBJS)
	@clang -o build/test-runtime$(EXE) $(OBJS) $(CFLAGS)

test: build/test-runtime$(EXE)
	@build/test-runtime$(EXE)
