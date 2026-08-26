#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

command -v cargo-audit >/dev/null || { echo 'cargo-audit is required: cargo install cargo-audit --locked' >&2; exit 2; }
command -v cargo-deny >/dev/null || { echo 'cargo-deny is required: cargo install cargo-deny --locked' >&2; exit 2; }

cargo audit
cargo deny check advisories licenses bans sources
