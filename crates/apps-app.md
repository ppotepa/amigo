# apps-app

Path: `crates/apps/app`  
Cargo package: `amigo-app`  
Layer: application host

## Role

Thin runtime host, bootstrap seam, platform/window orchestration, and presentation entrypoint.

## Owns

- startup/bootstrap seams
- platform/window host wiring
- runtime preset selection
- top-level lifecycle glue

## Does not own

- legacy/v2 parallel paths
- large formatting-only diffs
- silent fallback behavior
- domain guessing outside owner
- domain semantics
- render extraction policy
- plugin-specific behavior
- scene component implementation

## Important files found in snapshot

- `crates/apps/app/src/app_helpers.rs`
- `crates/apps/app/src/app_model/display.rs`
- `crates/apps/app/src/app_model/options.rs`
- `crates/apps/app/src/app_model/summary.rs`
- `crates/apps/app/src/assets/mod.rs`
- `crates/apps/app/src/bootstrap.rs`
- `crates/apps/app/src/debug_overlay/mod.rs`
- `crates/apps/app/src/diagnostics.rs`
- `crates/apps/app/src/event_pipeline.rs`
- `crates/apps/app/src/host_runtime.rs`
- `crates/apps/app/src/launch_selection.rs`
- `crates/apps/app/src/lib.rs`
- `crates/apps/app/src/main.rs`
- `crates/apps/app/src/orchestration/audio_bridge.rs`
- `crates/apps/app/src/orchestration/console_bridge.rs`
- `crates/apps/app/src/orchestration/mod.rs`
- `crates/apps/app/README.md`

## Dependencies seen in Cargo.toml

- `amigo-app-host-api`
- `amigo-app-host-winit`
- `amigo-assets`
- `amigo-camera-core-plugin`
- `amigo-camera-optics-plugin`
- `amigo-capabilities`
- `amigo-core`
- `amigo-devtools`
- `amigo-editor-authoring`
- `amigo-editor-ingame`
- `amigo-file-watch-api`
- `amigo-font`
- `amigo-hot-reload`
- `amigo-input-api`
- `amigo-math`
- `amigo-modding`
- `amigo-overlay-api`
- `amigo-render-api`
- `amigo-render-wgpu`
- `amigo-runtime`
- `amigo-runtime-bundles`
- `amigo-runtime-control`
- `amigo-scene`
- `amigo-scripting-api`
- `amigo-session`
- `amigo-state`
- `amigo-window-api`
- `serde_yaml`

## Documentation status

README present: `true`

If this crate is touched, keep documentation close to the touched ownership boundary. Do not use this crate doc as permission to perform broad cleanup.

## Allowed changes

```text
small changes inside crate ownership
contract changes with downstream validation
local tests for crate-owned behavior
diagnostics that expose missing contracts or invalid input
```

## Forbidden changes

```text
cross-layer behavior leaks
legacy/v2 duplicate paths
large formatting-only rewrites
new hidden fallback behavior
```

## Validation commands

```powershell
cargo check -p amigo-app
cargo test -p amigo-app --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/apps/app
rg -n "pub struct|pub enum|pub trait|impl " crates/apps/app/src
```
