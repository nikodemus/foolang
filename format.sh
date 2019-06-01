#!/usr/bin/env bash
rustfmt --edition 2018 $(git ls-files | grep -E "\.rs$")
