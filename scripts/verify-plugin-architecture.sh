#!/usr/bin/env bash
set -euo pipefail

cargo test -p amigo-plugin-api
cargo test -p amigo-codemap-api
cargo test -p amigo-plugin-index
cargo test -p amigo-plugin-manifest
cargo test -p amigo-plugin-loader
cargo run -p amigo-plugin-check -- summary plugins
cargo run -p amigo-plugin-check -- plugins plugins
cargo run -p amigo-plugin-check -- targets plugins
cargo run -p amigo-plugin-check -- diagnostics plugins

if [[ "${AMIGO_VERIFY_WORKSPACE:-0}" == "1" ]]; then
  cargo check --workspace
  cargo test --workspace
fi

if rg "legacy|deprecated|_v2" plugins crates/apps/app crates/runtime crates/engine/scene crates/engine/render-wgpu crates/scripting/rhai crates/engine/devtools mods \
  --glob '!**/target/**' \
  --glob '!**/*.md' \
  --glob '!**/tests/**'; then
  echo "forbidden migration naming found"
  exit 1
fi

if rg "luma_fallback|guess_optical|direct_lens_flare" crates plugins crates/apps mods \
  --glob '!**/target/**' \
  --glob '!**/*.md'; then
  echo "forbidden renderer/domain heuristic found"
  exit 1
fi

if rg "pub use .*legacy|pub use .*camera_optical|pub use .*lens_artifact" crates plugins crates/apps \
  --glob '!**/target/**' \
  --glob '!**/*.md'; then
  echo "forbidden compatibility re-export found"
  exit 1
fi

echo "plugin architecture verification passed"
