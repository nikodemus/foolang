#!/usr/bin/env bash
#
# Foolang bootstrap script
#
# 1. Build the bootstrap interpreter.
# 2. Build the current compiler from sources using the bootstrap interpreter
# 3. Build the current cmopiler with itself
# 4. Build the current compiler with compiler from step 3.
# 5. Verify that results from steps 3. and 4. are identical.
#
# This takes around 10min. Keep myself entertained by printing stuff
# in pretty colors.
#
set -euo pipefail

source build_utils.sh

mkdir -p build/
BOOTSTRAP_INTERPRETER=$(exename target/debug/bootstrap-interpreter)
BOOTSTRAP_COMPILER=$(exename build/bootstrap-foo$)
SELF_COMPILER=$(exename build/self-compiler)
FOO=$(exename build/foo)

echo "Expected bootstrap time: ~5-10min"

write_build_info
on_exit restore_build_info
on_exit_if_tty play_beep

if [[ "$@" != "--skip-interpreter-build" ]]; then
    start_clock "bootstrap interpreter"
    if cargo build &> build/bootstrap-interpreter.log; then
        ok
    else
        fail build/bootstrap-interpreter.log
    fi
fi

rm -rf build/bootstrap-compiler-c
start_clock "bootstrap compiler"
if $BOOTSTRAP_INTERPRETER foo/foo.foo -- --compile foo/foo.foo \
    &> build/bootstrap-compiler.log
then
    ok
    mv $(exename foo/foo) $BOOTSTRAP_COMPILER
    cp -a c build/bootstrap-compiler-c
else
    fail build/bootstrap-compiler.log
fi

rm -rf build/self-compiler-c
start_clock "self-compiler"
if $BOOTSTRAP_COMPILER --compile foo/foo.foo \
    &> build/self-compiler.log
then
    ok
    mv $(exename foo/foo) $SELF_COMPILER
    cp -a c build/self-compiler-c
else
    fail build/self-compiler.log
fi

rm -rf build/foo-c
start_clock "foolang"
if $SELF_COMPILER --compile foo/foo.foo \
    &> build/foo.log
then
    ok
    mv $(exename foo/foo) $FOO
    cp -a c build/foo-c
else
    fail build/foo.log
fi

if diff --recursive --brief build/foo-c build/self-compiler-c \
    &> build/bootstrap-diff.log
then
    echo "Bootstrap complete, enjoy quietly!"
else
    echo "***********************************"
    echo "* $(red WARNING: INCONSISTENT BOOTSTRAP) *"
    echo "***********************************"
    echo -n "Host: "
    uname -a
    echo -n "Foolang: "
    git describe --tags --dirty
    echo
    echo "Please report the above to: https://github.com/nikodemus/foolang"
    echo
    echo "Anyhow, the binary is *probably* fine. Enjoy quietly!"
fi
echo "Binary: $FOO"
