#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo '==> cargo fmt'
cargo fmt --all -- --check

echo '==> plugin contracts'
cargo run -p amigo-plugin-check -- validate --workspace --plugins plugins

echo '==> architecture dependencies'
python3 scripts/architecture-lint.py

echo '==> workspace check'
cargo check --workspace --all-targets

echo '==> clippy critical crates'
cargo clippy \
  -p amigo-runtime \
  -p amigo-plugin-api \
  -p amigo-plugin-index \
  -p amigo-plugin-loader \
  -p amigo-render-api \
  -p amigo-scripting-rhai \
  --all-targets -- -D warnings

echo '==> contract tests'
cargo test \
  -p amigo-plugin-api \
  -p amigo-plugin-index \
  -p amigo-plugin-loader \
  -p amigo-render-api \
  -p amigo-scripting-rhai
