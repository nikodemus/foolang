#!/bin/bash

if [ -z "$WINDIR" ]; then
    ostype="unix"
else
    ostype="windows"
fi

set -euo pipefail

# This tricks clang into using the internal symbolizer, leaving path
# empty leaves the addresses unsymbolized.
# export ASAN_SYMBOLIZER_PATH=0

rm -f tmp_transpile_test.exe

function stacksize_options() {
    # 4MB stack: the parser is highly recursive!
    local size=0x400000
    if [ $ostype = "unix" ]; then
        echo -Wl,-z,-stack-size=$size
    else
        echo -Wl,/STACK:$size
    fi
}

clang \
    -o tmp_transpile_test.exe \
    $(stacksize_options) \
    -lm -fno-omit-frame-pointer -g -Wall --std=c11 \
    c/main.c c/system_$ostype.c ext/dtoa.c
echo "Built, running!"
time valgrind ./tmp_transpile_test.exe "$@"
