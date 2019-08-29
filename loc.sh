#!/bin/bash

set -euo pipefail

cd $(dirname $0)

git ls-files \
    | grep -E '\.rs$' \
    | grep -vE '^src/tests|^tests|^src/bin/bench.rs$' \
    | xargs wc -l \
    | awk 'END { print "Rust code " $1 }'

git ls-files \
    | grep -E '\.rs$' \
    | grep -E '^src/tests|^tests|^src/bin/bench.rs$' \
    | xargs wc -l \
    | awk 'END { print "Rust test " $1 }'

git ls-files \
    | grep -E '.foo$' \
    | xargs wc -l \
    | awk 'END { print "Foolang   " $1 }'

git ls-files \
    | grep -E '.(html|css|js)$' \
    | grep -vE '^ide/lib' \
    | xargs wc -l \
    | awk 'END { print "Javacript " $1 }'
