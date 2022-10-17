#!/usr/bin/env bash

set -euo pipefail

install_el() {
    local el=$(basename $1)
    if ! [ -f elisp/$el ]; then
        curl $1 -o elisp/$el
    fi
}

install_el https://raw.githubusercontent.com/Fanael/parent-mode/master/parent-mode.el
install_el https://raw.githubusercontent.com/Fanael/highlight-numbers/master/highlight-numbers.el

# First without highlight-numbers
emacs -Q --chdir elisp --batch \
    --load foolang.el

# Then with
emacs -Q --chdir elisp --batch \
    --load parent-mode.el \
    --load highlight-numbers.el \
    --load foolang.el

rm elisp/parent-mode.el elisp/highlight-numbers.el
