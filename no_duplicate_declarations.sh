#!/usr/bin/env bash
set -euo pipefail

if [ "-$@" = "---build" ]; then
    cargo run foo/compile.foo -- foo/examples/hello.foo hello.exe
fi

sort c/generated_declarations.h > sorted_declarations.tmp
uniq sorted_declarations.tmp > unique_declarations.tmp

if ! diff -u unique_declarations.tmp sorted_declarations.tmp; then
    echo "---"
    echo "DUPLICATE DECLARATIONS FOUND"
    exit 1
fi
