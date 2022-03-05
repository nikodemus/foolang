#!/bin/bash

if [ -z "$WINDIR" ]; then
    ostype="unix"
else
    ostype="windows"
fi

set -euo pipefail

export ASAN_SYMBOLIZER_PATH=llvm-symbolizer

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
    -fsanitize=address -fsanitize=undefined \
    -fno-omit-frame-pointer -g -Wall --std=c11 \
    c/main.c c/system_$ostype.c ext/dtoa.c c/mark-and-sweep.c
echo "Built, running!"
time ./tmp_transpile_test.exe "$@"
