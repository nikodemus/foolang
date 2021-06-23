#!/bin/bash

if [ -z "$WINDIR" ]; then
    ostype="unix"
else
    ostype="windows"
fi

set -euo pipefail

# This tricks clang into using the internal symbolizer, leaving path
# empty leaves the addresses unsymbolized.
export ASAN_SYMBOLIZER_PATH=0

rm -f tmp_transpile_test.exe

clang \
    -o tmp_transpile_test.exe \
    -fsanitize=address -fsanitize=undefined \
    -fno-omit-frame-pointer -g -Wall --std=c11 \
    host/main.c host/system_$ostype.c ext/dtoa.c
echo "Built, running!"
time ./tmp_transpile_test.exe "$@"
