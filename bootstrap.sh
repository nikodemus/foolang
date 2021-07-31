#!/usr/bin/env bash
set -eo pipefail

if [ -z "$WINDIR" ]; then
    EXT=""
else
    EXT=".exe"
fi

set -u

mkdir -p bin/
BOOTSTRAP_COMPILER=bin/bootstrap-fooc$EXT
TARGET_COMPILER=bin/fooc$EXT
TARGET_FOO=bin/foo$EXT

trap "./beep.sh" EXIT

# Overwrite default "unknown" with git version.
V=foo/lang/build_info.foo
echo 'define Version'                   > $V
echo '    "'$(git describe --tags)'"!' >> $V

trap "git checkout HEAD foo/lang/build_info.foo" EXIT

echo "Building $BOOTSTRAP_COMPILER..."
time cargo run -- foo/compile.foo -- foo/compile.foo $BOOTSTRAP_COMPILER
rm -rf bootstrap-c
cp -a c bootstrap-c
echo "$BOOTSTRAP_COMPILER built!"

echo "Building $TARGET_COMPILER..."
time $BOOTSTRAP_COMPILER foo/compile.foo $TARGET_COMPILER
rm -rf target-compiler-c
cp -a c target-compiler-c
echo "$TARGET_COMPILER built!"

echo "Building $TARGET_FOO..."
time $TARGET_COMPILER foo/foo.foo $TARGET_FOO
rm -rf target-foo-c
cp -a c target-foo-c
echo "$TARGET_FOO built!"
