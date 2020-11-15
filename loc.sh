#!/bin/bash

set -euo pipefail

cd $(dirname $0)

TOTAL=0

count() {
    local where=$1
    local name=$2
    local lines=$(eval git ls-files $where | $name | xargs wc -l | awk 'END { print $1 }')
    echo "$name $lines" | sed -s 's|_|/|g'
    TOTAL=$((TOTAL+lines))
}

Rust_code() {
    grep -E '\.rs$' \
        | grep -vE '^src/tests|^tests|^src/bin/bench.rs$'
}

Rust_test() {
    grep -E '\.rs$' \
        | grep -E '^src/tests|^tests|^src/bin/bench.rs$'
}

Foolang() {
    grep -E '.foo$'
}

C() {
    grep -E '.c$'
}

Elisp() {
    grep -E '.el$'
}

Markdown() {
    grep -E '.(md)$'
}

count foo   "Foolang  "
count docs  "Markdown "
count elisp "Elisp    "
count host  "C        "
count .     "Rust_code"
count .     "Rust_test"
echo        "---------------"
echo        "    Total" $TOTAL
