#!/usr/bin/env bash
set -euo pipefail

# Stick
#   ; ./beep 3
# after a self-built, to let you know when it's done.

play_beep() {
    echo -en "\007"
}

if [ -z "$@" ]; then
    play_beep
else
    N=$1
    while [ $N -gt 0 ]; do
        play_beep
        sleep 2
        N=$((N-1))
    done
fi
