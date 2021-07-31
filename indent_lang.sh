#!/bin/bash

set -euo pipefail

git checkout foo/lang/

F=${1:-any.foo}

while true
do
    cargo run foo/foo.foo -- --format-in-place foo/lang/$F || break
    git diff --exit-code foo/lang/$F || break
    F=$(ls -1 foo/lang/ | grep -A1 ^$F | tail -n 1)
done
