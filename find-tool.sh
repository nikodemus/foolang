#!/bin/bash
#
# Find both clang and llvm-ar with the same version.
#
# Tries first without an explicit version, then down from
# 14 to 6.

set -euo pipefail

if [ -z "$@" ]; then
    echo "ERROR: find-tool.sh needs an argument: --cc, --ar, or --debug" 2>&1
    exit 1
fi

debug=false

AR_TOOLS="llvm-ar llvm-ar-14 llvm-ar-13 llvm-ar-12 llvm-ar-11 llvm-ar-10 \
    gcc-ar gcc-ar-12 gcc-ar-11 gcc-ar-10 gcc-ar-9 \
    ar"
CC_TOOLS="clang clang-14 clang-13 clang-12 clang-11 clang-10 \
    gcc gcc-12 gcc-11 gcc-10 gcc-9 \
    cc"

list_path() {
    IFS=:
    for dir in $PATH; do
        echo "---"
        echo "PATH: $dir"
        ls -C $dir || true
    done
}

find_tool() {
    for tool in $@; do
        if $tool --version &> /dev/null; then
            if $debug; then
                echo "Found: $tool"
            else
                echo $tool
            fi
            return
        else
            if $debug; then
                echo "Not found: $tool"
            fi
            continue
        fi
    done
}

case $1 in
    --debug)
        debug=true
        find_tool $AR_TOOLS
        find_tool $CC_TOOLS
        list_path
        exit
        ;;
    --ar)
        find_tool $AR_TOOLS
        ;;
    --cc)
        find_tool $CC_TOOLS
        ;;
esac
