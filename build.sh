#!/usr/bin/env bash
USAGE=$(cat <<EOF
usage: build.sh [option*]

  Foolang build script.

  Stages:

    1. build the bootstrap compiler unless it already exists.
    2. build current compiler with the bootstrap compiler.
    3. build current compiler with compiler from stage 2.
    4. build current compiler with compiler from stage 3.
    5. verify that steps #3 and #4 result in identical builds.

    The stage 3 build always is the final artefact.

  Options:

    --bootstrap

        Force stage 1. even if bootstrap compiler already exists.

    --no-interpreter-build

        Skip building the bootstrap interpreter. This is for CI use only.

    --no-verify

        Skip steps #4 and #5.

EOF
)
set -euo pipefail

source utils.sh
write_build_info
on_exit restore_build_info

BOOTSTRAP_INTERPRETER=$(exename target/debug/bootstrap-interpreter)
BOOTSTRAP_COMPILER=$(exename build/bootstrap-compiler)
SELF_COMPILER=$(exename build/self-compiler)
FOO=$(exename build/foo)
FOO2=$(exename build/foo2)

CLEAN=false
BOOTSTRAP=false
NO_INTERPRETER_BUILD=false
VERIFY=true

for option in $@
do
    case $option in
        --help|-h)
            echo "$USAGE"
            exit
            ;;
        --bootstrap)
            BOOTSTRAP=true
            ;;
        --no-interpreter-build)
            NO_INTERPRETER_BUILD=true
            ;;
        --no-verify)
            # It's just easier to keep verify option this way around.
            VERIFY=false
            ;;
        *)
            echo "ERROR: unknown option to build.sh: $option"
            echo "$USAGE"
            exit 1
            ;;
    esac
done

mkdir -p build/

# Figure out if we need a bootstrap or not.
if ! $BOOTSTRAP  && ! [[ -e $BOOTSTRAP_COMPILER ]]; then
    echo "Bootstrap compiler not found, bootstrapping."
    BOOTSTRAP=true
fi

if $BOOTSTRAP && $VERIFY; then
    echo "Expected build time: ~7min (bootstrap, self-build, foo, verify)"
elif $BOOTSTRAP; then
    echo "Expected build time: ~6min (bootstrap, self-build, foo)"
elif $VERIFY; then
    echo "Expected build time: ~3min (self-build, foo, verify)"
else
    echo "Expected build time: ~1.5min (self-build, foo)"
fi

if $BOOTSTRAP; then

    if $NO_INTERPRETER_BUILD && [[ -e $BOOTSTRAP_INTERPRETER ]]; then
        echo "Skipping bootstrap interpreter build."
    else
        if $NO_INTERPRETER_BUILD; then
            echo "Bootstrap interpreter not found, build forced."
        fi
        start_clock "building bootstrap interpreter"
        if cargo build &> build/bootstrap-interpreter.log; then
            ok
        else
        fail build/bootstrap-interpreter.log
        fi
    fi

    rm -rf build/bootstrap-compiler-c
    start_clock "building bootstrap compiler"
    if $BOOTSTRAP_INTERPRETER foo/foo.foo -- --compile foo/foo.foo \
        &> build/bootstrap-compiler.log
    then
        ok
        mv $(exename foo/foo) $BOOTSTRAP_COMPILER
        cp -a c build/bootstrap-compiler-c
    else
        fail build/bootstrap-compiler.log
    fi

fi

rm -rf build/self-compiler-c
start_clock "building self-compiler"
if $BOOTSTRAP_COMPILER --compile foo/foo.foo \
    &> build/self-compiler.log
then
    ok
    mv $(exename foo/foo) $SELF_COMPILER
    cp -a c build/self-compiler-c
else
    fail build/self-compiler.log
fi

rm -rf build/foo-c
start_clock "building foo"
if $SELF_COMPILER --compile foo/foo.foo \
    &> build/foo.log
then
    ok
    mv $(exename foo/foo) $FOO
    cp -a c build/foo-c
else
    fail build/foo.log
fi

rm -rf build/foo2-c
if $VERIFY; then
    start_clock "building second generation foo"
    if $FOO --compile foo/foo.foo \
        &> build/foo2.log
    then
        ok
        mv $(exename foo/foo) $FOO2
        cp -a c build/foo2-c
    else
        fail build/foo2.log
    fi
    if diff -u --recursive build/foo-c build/foo2-c \
        &> build/self-build.diff
    then
        echo "Self-build complete, enjoy quietly!"
    else
        echo -n "$(red WARNING): inconsistent self-build!"
        echo
        echo "Please report with contents of build/self-build.diff to"
        echo
        echo "    https://github.com/nikodemus/foolang"
        echo
    fi
else
    echo "Unverified build, consistency not checked - enjoy quietly!"
fi

echo "Binary: $FOO"
