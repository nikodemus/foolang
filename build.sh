#!/usr/bin/env bash
USAGE=$(cat <<EOF
usage: build.sh [option*]

  Foolang build script.

  By default:
    1. build the bootstrap compiler.
    2. build current compiler with the bootstrap compiler.
    3. build current compiler with itself.
    4. verify that steps #2 and #3 result in identical builds.

  Options:

    --skip-interpreter-build

        Skip building the bootstrap interpreter. This is for CI use only.

    --no-bootstrap

        Don't build the bootstrap compiler if it already exists.

    --no-verify

        Skip steps #3 and #4.

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
NO_BOOTSTRAP=false
NO_INTERPRETER_BUILD=false
VERIFY=true

for option in $@
do
    case $option in
        --help|-h)
            echo "$USAGE"
            exit
            ;;
        --no-bootstrap)
            NO_BOOTSTRAP=true
            ;;
        --no-interpreter-build)
            NO_INTERPRETER_BUILD=true
            ;;
        --no-verify)
            # It's just easier to keep verify option this way around,
            # and the others negated.
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
if $NO_BOOTSTRAP && [[ -e $BOOTSTRAP_COMPILER ]]; then
    echo "Skipping bootstrap."
    BOOTSTRAP=false
else
    if $NO_BOOTSTRAP; then
        echo "Bootstrap compiler not found, bootstrap forced despite --no-bootstrap."
    fi
    BOOTSTRAP=true
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

    if $NO_INTERPRETER_BUILD && [[ -e $BOOTSTRAP_INTERPRETER ]]; then
        echo "Skipping bootstrap interpreter build"
    else
        if $NO_INTERPRETER_BUILD; then
            echo "Bootstrap interpreter not found: build forced despite --no-interpreter-build."
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
    if diff -u --recursive build/self-compiler-c build/second-generation-c \
        &> build/self-build.diff
    then
        echo "Self-build complete, enjoy quietly!"
    else
        echo -n "$(red WARNING): inconsistent self-build!"
        if $NO_BOOTSTRAP;
        then
            echo " Please try without --no-bootstrap option."
        else
            # Bootstrap wasn't intentionally skipped, something is borken.
            echo
            echo "Please report with contents of build/self-build.diff to"
            echo
            echo "    https://github.com/nikodemus/foolang"
            echo
        fi
    fi
else
    echo "Unverified build, consistency not checked - enjoy quietly!"
fi

echo "Binary: $SELF_COMPILER"
