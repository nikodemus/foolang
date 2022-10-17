.PHONY: all
all:
	@echo "targets:"
	@echo " - test (test-rust, test-elisp, test-foolang, test-cps,"
	@echo "   test-runtime, test-c-backend, test-benchmark)"
	@echo " - clean (clean-c, clean-rust)"
	@echo " - commit (tests and commits)"
	@echo " - amend (tests and amends last commit)"

.EXTRA_PREREQS:= $(abspath $(lastword $(MAKEFILE_LIST)))

CC := $(shell bash ./find-clang.sh cc)
AR := $(shell bash ./find-clang.sh ar)

ifeq ($(CC), "")
	$(error Could not find matching clang versions for CC and AR!)
endif

$(info Using CC = $(CC), AR = $(AR))

CPPFLAGS = -Iruntime -Iext
CFLAGS = -g -Wall -Wextra -fsanitize=address -fsanitize=undefined
DEPFLAGS = -MT $@ -MMD -MP -MF build/$*.d
BUILD.a = @$(AR) rc
BUILD.o = @$(CC) $(DEPFLAGS) $(CFLAGS) $(CPPFLAGS) -c
BUILD.exe = @$(CC) $(CFLAGS) $(CPPFLAGS)
SILENCE = | (grep -v "Creating library" || true)

LOG_BUILD = @echo Building: $@

ifeq ($(OS), Windows_NT)
	EXE=.exe
else
	EXE=.bin
endif

LIBFOO=build/libfoolang.a

RUNTIME_SRCS=$(wildcard runtime/*.c)
RUNTIME_OBJS=$(RUNTIME_SRCS:%.c=build/%.o)
RUNTIME_DEPFILES=$(RUNTIME_SRCS:%.c=build/%.d)
$(RUNTIME_OBJS): | build/runtime

# Files under tests/c-backend/ are code emitted by
# the C-backend.
#
# Emission tests check that we get the source we expect
# by comparing to the saved files.
#
# Run tests compile and run the saved files. This means
# re-running tests doesn't need to rebuild all tests unless
# the expected source (or runtime!) has changed.
C_BACKEND_TEST_SRCS=$(wildcard tests/c-backend/*.c)
C_BACKEND_TEST_OBJS=$(C_BACKEND_TEST_SRCS:%.c=build/%.o)
C_BACKEND_TEST_EXES=$(C_BACKEND_TEST_SRCS:%.c=build/%$(EXE))
C_BACKEND_TEST_RUNS=$(C_BACKEND_TEST_SRCS:%.c=build/%.run)
$(C_BACKEND_TEST_OBJS): | build/tests/c-backend

# Files under tests/runtime are unit tests for runtime.
RUNTIME_TEST_SRCS=$(wildcard tests/runtime/*.c)
RUNTIME_TEST_OBJS=$(RUNTIME_TEST_SRCS:%.c=build/%.o)
$(RUNTIME_TEST_OBJS): | build/tests/runtime

# Files under tests/bench are benchmark-like tests.
# Currently a hand-compiled version of pi.foo, and
# same expressed in "natural" C.
#
# Currently just run, not timed.
BENCHMARK_SRCS=$(wildcard tests/benchmark/*.c)
BENCHMARK_OBJS=$(BENCHMARK_SRCS:%.c=build/%.o)
BENCHMARK_EXES=$(BENCHMARK_SRCS:%.c=build/%$(EXE))
BENCHMARK_RUNS=$(BENCHMARK_SRCS:%.c=build/%.run)
$(BENCHMARK_OBJS): | build/tests/benchmark

build/runtime:
	@mkdir -p build/runtime

build/tests/c-backend:
	@mkdir -p build/tests/c-backend

build/tests/runtime:
	@mkdir -p build/tests/runtime

build/tests/benchmark:
	@mkdir -p build/tests/benchmark

$(LIBFOO): $(RUNTIME_OBJS)
	$(LOG_BUILD)
	$(BUILD.a) $@ $(RUNTIME_OBJS)

build/tests/runtime/test$(EXE): $(RUNTIME_TEST_OBJS) $(LIBFOO)
	$(LOG_BUILD)
	$(BUILD.exe) -o $@ $^ $(SILENCE)

.PRECIOUS: %$(EXE)
%$(EXE): %.o $(LIBFOO)
	$(LOG_BUILD)
	$(BUILD.exe) -o $@ $^ $(SILENCE)

build/%.o : %.c
	$(LOG_BUILD)
	$(BUILD.o) -o $@ $<

.PHONY: build/%.run
build/%.run: build/%$(EXE)
	@echo Running: $<
	@$<

.PHONY: test-benchmark
test-benchmark: $(BENCHMARK_RUNS)

.PHONY: test-c-backend
test-c-backend: $(C_BACKEND_TEST_RUNS)

.PHONY: test-runtime
test-runtime: build/tests/runtime/test.run

.PHONY: test-cps
test-cps:
	cargo run foo/impl/cps.foo --use=foo/lib

# Purposefully not running the old transpiler tests!
.PHONY: test-foolang
	cargo run foo/impl/test_foolang.foo --use=foo/lib

.PHONY: test-rust
test-rust:
	@cargo test

.PHONY: test-elisp
test-elisp:
	@tests/test-elisp.sh

.PHONY: test
test: test-rust test-elisp test-foolang
test: test-cps test-runtime test-c-backend test-benchmark

.PHONY: commit
commit: test
	@git commit -a -v

.PHONY: amend
amend: test
	@git commit -a --amend -v

.PHONY: clean
clean: clean-c clean-rust

.PHONY: clean-c
clean-c:
	@rm -rf build/runtime build/test build/libfoolang.a

.PHONY: clean-rust
clean-rust:
	@cargo clean

$(RUNTIME_DEPFILES):
include $(wildcard $(RUNTIME_DEPFILES))
