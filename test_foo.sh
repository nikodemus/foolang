#!/usr/bin/env bash

set -euo pipefail

run() {
    name=$1; shift
    if ! cargo run -- foo/impl/$name --use=foo/lib -- $@; then
        echo "FAIL: $name"
        exit 1
    fi
}

if [ -z "$@" ]; then
    run test_foolang.foo
    run test_prelude.foo
    run test_transpile.foo
else
    for test in "$@"; do
        run "test_$test.foo"
    done
fi
