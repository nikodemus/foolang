.PHONY: all
all:
	@echo "targets:"
	@echo " - test (test-rust, test-cps, test-c-backend, test-benchmark)"
	@echo " - clean (clean-c, clean-rust)"
	@echo " - commit (tests and commits)"
	@echo " - amend (tests and amends last commit)"

.EXTRA_PREREQS:= $(abspath $(lastword $(MAKEFILE_LIST)))

CC := $(shell bash ./find-clang.sh cc)
AR := $(shell bash ./find-clang.sh ar)

ifeq ($(CC), "")
	$(error Could not clang versions for CC and AR!)
endif

$(info Using CC = $(CC), AR = $(AR))

CPPFLAGS = -Iruntime -Iext
CFLAGS = -g -Wall -Wextra -fsanitize=address -fsanitize=undefined
DEPFLAGS = -MT $@ -MMD -MP -MF build/$*.d
COMPILE.a = "$(AR)" rc
COMPILE.c = "$(CC)" $(DEPFLAGS) $(CFLAGS) $(CPPFLAGS) -c
COMPILE.exe = "$(CC)" $(DEPFLAGS) $(CFLAGS) $(CPPFLAGS)
SILENCE = | (grep -v "Creating library" || true)

LOG_BUILD = @echo Building: $@

ifeq ($(OS), Windows_NT)
	EXE=.exe
else
	EXE=
endif

RUNTIME_SRCS=$(wildcard runtime/*.c)
RUNTIME_OBJS=$(RUNTIME_SRCS:%.c=build/%.o)
RUNTIME_DEPFILES=$(RUNTIME_SRCS:%.c=build/%.d)
$(RUNTIME_OBJS): | build/runtime

# Files under test/c-backend/ are code emitted by
# the C-backend.
#
# Emission tests check that we get the source we expect
# by comparing to the saved files.
#
# Run tests compile and run the saved files. This means
# re-running tests doesn't need to rebuild all tests unless
# the expected source (or runtime!) has changed.
C_BACKEND_TEST_SRCS=$(wildcard test/c-backend/*.c)
C_BACKEND_TEST_OBJS=$(C_BACKEND_TEST_SRCS:%.c=build/%.o)
C_BACKEND_TEST_EXES=$(C_BACKEND_TEST_SRCS:%.c=build/%$(EXE))
C_BACKEND_TEST_RUNS=$(C_BACKEND_TEST_SRCS:%.c=build/%.run)
$(C_BACKEND_TEST_OBJS): | build/test/c-backend

# Files under test/runtime are unit tests for runtime.
RUNTIME_TEST_SRCS=$(wildcard test/runtime/*.c)
RUNTIME_TEST_OBJS=$(RUNTIME_TEST_SRCS:%.c=build/%.o)
$(RUNTIME_TEST_OBJS): | build/test/runtime

# Files under test/bench are benchmark-like tests.
# Currently a hand-compiled version of pi.foo, and
# same expressed in "natural" C.
#
# Currently just run, not timed.
BENCHMARK_SRCS=$(wildcard test/benchmark/*.c)
BENCHMARK_OBJS=$(BENCHMARK_SRCS:%.c=build/%.o)
BENCHMARK_EXES=$(BENCHMARK_SRCS:%.c=build/%$(EXE))
BENCHMARK_RUNS=$(BENCHMARK_SRCS:%.c=build/%.time)
$(BENCHMARK_OBJS): | build/test/benchmark

build/runtime:
	@mkdir -p build/runtime

build/test/c-backend:
	@mkdir -p build/test/c-backend

build/test/runtime:
	@mkdir -p build/test/runtime

build/test/benchmark:
	@mkdir -p build/test/benchmark

build/foolang.a: $(RUNTIME_OBJS)
	$(LOG_BUILD)
	@$(COMPILE.a) $(OUTPUT_OPTION) $(RUNTIME_OBJS)

build/test/runtime/test$(EXE): $(RUNTIME_TEST_OBJS) build/foolang.a
	$(LOG_BUILD)
	@$(COMPILE.exe) $(OUTPUT_OPTION) $^ $(SILENCE)

.PRECIOUS: %$(EXE)
%$(EXE): build/foolang.a %.o
	$(LOG_BUILD)
	@$(COMPILE.exe) $(OUTPUT_OPTION) $^ $(SILENCE)

build/%.o : %.c Makefile
	$(LOG_BUILD)
	@$(COMPILE.c) $(OUTPUT_OPTION) $<

.PHONY: build/%.run
build/%.run: build/%$(EXE)
	@echo Running: $^
	@$<

.PHONY: build/%.time
build/%.time: build/%$(EXE)
	@echo Timing: $^
	@bash -c "echo ' '`(time ($< &> /dev/null)) 2>&1 | grep real`"

.PHONY: test-benchmark
test-benchmark: $(BENCHMARK_RUNS)

.PHONY: test-c-backend
test-c-backend: $(C_BACKEND_TEST_RUNS)

.PHONY: test-runtime
test-runtime: build/test/runtime/test.run

.PHONY: test-cps
test-cps:
	cargo run foo/impl/cps.foo --use=foo/lib

# Purposefully not running the old transpiler tests!
.PHONY: test-foolang
	cargo run foo/impl/test_foolang.foo --use=foo/lib

.PHONY: test-rust
test-rust:
	@cargo test

.PHONY: test
test: test-rust test-foolang test-cps test-runtime test-c-backend test-benchmark

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
	@rm -rf build/runtime build/test build/foolang.a

.PHONY: clean-rust
clean-rust:
	@cargo clean

$(RUNTIME_DEPFILES):
include $(wildcard $(RUNTIME_DEPFILES))
