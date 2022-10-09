.PHONY: all
all:
	@echo "targets:"
	@echo " - test (unit tests for new runtime & CPS compiler)"
	@echo " - clean (delete new runtime objects)"
	@echo " - commit (tests and commits)"
	@echo " - amend (tests and amends last commit)"

CC := clang
CPPFLAGS := -I.
CFLAGS := -g -Wall -Wextra -fsanitize=address -fsanitize=undefined
DEPFLAGS = -MT $@ -MMD -MP -MF build/$*.d
COMPILE.c = $(CC) $(DEPFLAGS) $(CFLAGS) $(CPPFLAGS) -c

ifeq ($(OS), Windows_NT)
	EXE=.exe
else
	EXE=
endif

SRCS=$(wildcard runtime/*.c)
OBJS=$(SRCS:%.c=build/%.o)

$(OBJS): | build/runtime

build/runtime:
	@mkdir -p build/runtime

build/%.o : %.c Makefile
	@echo - $@
	@$(COMPILE.c) $(OUTPUT_OPTION) $<

DEPFILES := $(SRCS:%.c=build/%.d)
$(DEPFILES):

include $(wildcard $(DEPFILES))

build/test-runtime$(EXE): $(OBJS)
	@clang -o $@ $(OBJS) $(CFLAGS)

.PHONY: test
test: test-cps test-runtime

.PHONY: test-cps
test-cps:
	cargo run foo/impl/cps.foo --use=foo/lib

.PHONY: test-runtime
test-runtime: build/test-runtime$(EXE)
	@$<

.PHONY: commit
commit: test
	@git commit -a -v

.PHONY: amend
amend: test
	@git commit -a --amend -v

.PHONY: clean
clean:
	@rm -rf build/runtime build/test-runtime*
