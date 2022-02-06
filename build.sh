#!/usr/bin/env bash
#
# Foolang build script
#
# 1. If the bootstrap compiler doesn't exist, runs full bootstrap instead.
# 2. Builds the current compiler with itself.
#
# This takes around ~2min. Print the same pretty colors as full bootstrap
# does.
#
set -euo pipefail

source build_utils.sh

mkdir -p build/
BOOTSTRAP_COMPILER=$(exename build/bootstrap-compiler)
FOO=$(exename build/foo)

if [[ ! -e $BOOTSTRAP_COMPILER ]]; then
    echo "Bootstrap compiler not found: bootstrapping!"
    exec ./bootstrap.sh
fi

echo "Expected build time: ~1-2min"

on_exit_if_tty play_beep

write_build_info
on_exit restore_build_info

# Nuke both self-compiler and foo -stuff to avoid confusion.
rm -rf build/foo-c build/self-compiler-c build/self-compiler.log build/foo.log
start_clock "foolang"
if $BOOTSTRAP_COMPILER --compile foo/foo.foo \
    &> build/foo.log
then
    ok
    mv $(exename foo/foo) $FOO
    cp -a c build/foo-c
else
    fail build/foo.log
fi

echo "Enjoy quietly!"
echo "Binary: $FOO"
