#!/bin/bash
#
# Find both clang and llvm-ar with the same version.
#
# Tries first without an explicit version, then down from
# 14 to 6.

set -euo pipefail

if [ -z "$@" ]; then
    output=cc
else
    output=$1
fi

for v in "" -14 -13 -12 -11 -10 -9 -8 -7 -6; do
    cc=$(which clang$v || echo)
    ar=$(which llvm-ar$v || echo)
    [ -n "$cc" ] || continue
    [ -n "$ar" ] || continue
    echo ${!output}
    exit 0
done

echo "ERROR - could not find both clang and llvm-ar!" 1>&2
exit 1

    
    
