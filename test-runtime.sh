#!/usr/bin/env bash
set -euo pipefail

source utils.sh

TEST=$(exename build/test-runtime)

clang -o $TEST runtime/*.c -I. -Wall -Wextra
$TEST
