#!/usr/bin/env sh
set -eu

exec cargo run --quiet -p amigo-codemap -- "$@"
