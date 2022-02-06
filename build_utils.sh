#
# Intended to be sourced from other scripts.
#
# Contains common utilities for bootstrap.sh and build.sh
#

function green() {
    echo -e "\033[1;32;40m"$@"\033[0m"
}

function red() {
    echo -e "\033[1;31;40m"$@"\033[0m"
}

function start_clock() {
    BUILD_STEP_START=$(date +%s)
    echo -n "Building $1..."
}

function ok() {
    local STOP=$(date +%s)
    if [ -t 1 ]; then
        echo -e -n "\b\b\b [$(green ok)]"
    else
        echo -n " ok!"
    fi
    echo " ($((STOP-BUILD_STEP_START))s)"
}

function fail() {
    local STOP=$(date +%s)
    if [ -t 1 ]; then
        echo -e -n "\b\b\b [$(red FAILED)]"
    else
        echo -n " FAILED!"
    fi
    echo " ($((STOP-BUILD_STEP_START))s)"
    echo "--build log start--"
    cat $1
    echo "--build log end--"
    exit 1
}

EXIT_HOOKS=""

function run_exit_hooks() {
    for hook in $EXIT_HOOKS; do
        eval "$hook" || true
    done
}

function on_exit() {
    EXIT_HOOKS="$EXIT_HOOKS $@"
    trap run_exit_hooks EXIT
}

function on_exit_if_tty() {
    if [ -t 1 ]; then
        on_exit "$@"
    fi
}

function play_beep() {
    echo -en "\007"
}

function write_build_info() {
    # Overwrite default "unknown-version" with git describe
    local v=foo/lang/build_info.foo
    echo '---'                                                           > $v
    echo 'Generated by build_utils.sh, DO NOT COMMIT! Replaced by the'  >> $v
    echo 'default "unknown-version" once the build has finished.'       >> $v
    echo '---'                                                          >> $v
    echo 'define Version'                  >> $v
    echo '    "'$(git describe --tags --dirty 2> /dev/null || echo "version-info-missing")'"!' >> $v
}

function restore_build_info() {
    git checkout --quiet HEAD foo/lang/build_info.foo
}

function exename() {
    if [ -z "${WINDIR:-}" ]; then
        echo "$1"
    else
        echo "$1.exe"
    fi
}
