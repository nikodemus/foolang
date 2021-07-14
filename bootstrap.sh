#!/usr/bin/env bash
set -eo pipefail

if [ -z "$WINDIR" ]; then
    EXT=""
else
    EXT=".exe"
fi

set -u

BOOTSTRAP_COMPILER=./bootstrap-fooc$EXT
TARGET_COMPILER=./fooc$EXT

trap "./beep.sh" EXIT

echo "Building $BOOTSTRAP_COMPILER..."
time cargo run -- foo/compile.foo -- foo/compile.foo $BOOTSTRAP_COMPILER
rm -rf bootstrap-c
cp -a c bootstrap-c
echo "$BOOTSTRAP_COMPILER built!"

echo "Building $TARGET_COMPILER..."
time $BOOTSTRAP_COMPILER foo/compile.foo $TARGET_COMPILER
rm -rf target-c
cp -a c target-c
echo "$TARGET_COMPILER built!"
