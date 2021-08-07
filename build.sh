#!/usr/bin/env bash
USAGE=$(cat <<EOF
usage: build.sh [option*]

  Foolang build script.

  By default:
    1. build the bootstrap compiler unless it is already built.
    2. build current compiler with the bootstrap compiler.
    3. build current compiler with itself.
    4. verify that steps #2 and #3 result in identical builds.

  Options:

    --bootstrap

        Build bootstrap compiler build even if it already exists.

    --bootstrap-skip-interpreter-build

        Build bootstrap compiler build even if it already exists, but skip
        building the bootstrap interpreter. This is for CI use.

    --clean

        Delete build directory first. Forces full bootstrap.

    --no-verify

        Skip steps #3 and #4 for a faster build. Don't use this option when
        working on the compiler.

EOF
)
set -euo pipefail

source build_utils.sh
write_build_info
on_exit restore_build_info
on_exit_if_tty play_beep

BOOTSTRAP_INTERPRETER=$(exename target/debug/bootstrap-interpreter)
BOOTSTRAP_COMPILER=$(exename build/bootstrap-compiler)
SELF_COMPILER=$(exename build/foo)
SECOND_GENERATION=$(exename build/second-generation-foo)

CLEAN=false
FORCE_BOOTSTRAP=false
SKIP_INTERPRETER_BUILD=false
VERIFY=true

for option in $@
do
    case $option in
        --help|-h)
            echo "$USAGE"
            exit
            ;;
        --bootstrap)
            FORCE_BOOTSTRAP=true
            ;;
        --bootstrap-*)
            FORCE_BOOTSTRAP=true
            SKIP_INTERPRETER_BUILD=true
            ;;
        --clean)
            CLEAN=true
            ;;
        --no-verify)
            VERIFY=false
            ;;
        *)
            echo "ERROR: unknown option to build.sh: $option"
            echo "$USAGE"
            exit 1
            ;;
    esac
done

if $CLEAN; then
    rm -rf build/
fi
mkdir -p build/

if $FORCE_BOOTSTRAP || $CLEAN; then
    BOOTSTRAP=true
elif ! [[ -e $BOOTSTRAP_COMPILER ]]; then
    echo "Bootstrap compiler not found."
    BOOTSTRAP=true
else
    BOOTSTRAP=false
fi

if $BOOTSTRAP && $VERIFY; then
    echo "Expected build time: ~8min (bootstrap, self-build, verify)"
elif $BOOTSTRAP; then
    echo "Expected build time: ~6min (bootstrap, self-build)"
elif $VERIFY; then
    echo "Expected build time: ~4min (self-build, verify)"
else
    echo "Expected build time: ~2min (self-build)"
fi

if $BOOTSTRAP; then

    if $SKIP_INTERPRETER_BUILD && [[ -e $BOOTSTRAP_INTERPRETER ]]; then
        echo "Skipping bootstrap interpreter build"
    else
        if $SKIP_INTERPRETER_BUILD; then
            echo "Bootstrap interpreter not found: cannot skip."
        fi
        start_clock "bootstrap interpreter"
        if cargo build &> build/bootstrap-interpreter.log; then
            ok
        else
        fail build/bootstrap-interpreter.log
        fi
    fi

    rm -rf build/bootstrap-compiler-c
    start_clock "bootstrap compiler"
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
start_clock "self-compiler"
if $BOOTSTRAP_COMPILER --compile foo/foo.foo \
    &> build/self-compiler.log
then
    ok
    mv $(exename foo/foo) $SELF_COMPILER
    cp -a c build/self-compiler-c
else
    fail build/self-compiler.log
fi

rm -rf build/second-generation-c
if $VERIFY; then
    start_clock "second generation self-compiler"
    if $SELF_COMPILER --compile foo/foo.foo \
        &> build/foo.log
    then
        ok
        mv $(exename foo/foo) $SECOND_GENERATION
        cp -a c build/second-generation-c
    else
        fail build/foo.log
    fi
    if diff --recursive --brief build/second-generation-c build/self-compiler-c \
        &> build/bootstrap-diff.log
    then
        echo "Self-build complete, enjoy quietly!"
    else
        echo "***********************************"
        echo "* $(red WARNING: INCONSISTENT SELF-BUILD) *"
        echo "***********************************"
        echo -n "Host: "
        uname -a
        echo -n "Foolang: "
        git describe --tags --dirty 2> /dev/null || echo "version-info-missing"
        echo
        echo "Please report the above to: https://github.com/nikodemus/foolang"
        echo
        echo "Anyhow, the binary is *probably* fine. Enjoy quietly!"
    fi
else
    echo "Unverified build, consistency not checked - enjoy quietly!"
fi

echo "Binary: $SELF_COMPILER"
