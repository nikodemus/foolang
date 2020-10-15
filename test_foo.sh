#!/usr/bin/env bash

set -euo pipefail

run() {
    if ! cargo run -- foo/impl/$1 --use=foo/lib; then
        echo "FAIL: $1"
        exit 1
    fi
}

run test_foolang.foo
run test_prelude.foo
run test_transpile.foo
