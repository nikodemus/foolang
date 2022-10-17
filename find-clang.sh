#!/bin/bash
#
# Find both clang and llvm-ar with the same version.
#
# Tries first without an explicit version, then down from
# 14 to 6.

set -euo pipefail

if [ -z "$@" ]; then
    echo "ERROR: find-clang.sh needs an argument: --cc, --ar, or --debug" 2>&1
    exit 1
fi

case $1 in
    --debug)
        IFS=:
        for dir in $PATH; do
            echo "---"
            echo "PATH: $dir"
            ls -C $dir || true
        done
        exit
        ;;
    --ar)
        output=ar
        ;;
    --cc)
        output=cc
        ;;
esac

for v in "" -14 -13 -12 -11 -10 -9 -8 -7 -6; do
    cc=clang$v
    ar=llvm-ar$v
    $cc --version &> /dev/null || continue
    $ar --version &> /dev/null || continue
    echo ${!output}
    exit 0
done

exit 1
