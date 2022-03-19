#!/usr/bin/env bash

set -euo pipefail

source utils.sh

FOO=$(exename build/foo)

start_clock "Running interpreted tests"
if $FOO foo/tests/self_test.foo &> test_foo.log
then
    ok
    rm test_foo.log
else
    fail test_foo.log
fi

start_clock "Compiling tests"
if $FOO --compile foo/tests/self_test.foo &> test_foo.log
then
    ok
    rm test_foo.log
else
    fail test_foo.log
fi

start_clock "Running compiled tests"
if $(exename foo/tests/self_test) &> test_foo.log
then
    ok
    rm test_foo.log
else
    fail test_foo.log
fi
