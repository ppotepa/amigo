# runtime-bundles

Path: `crates/runtime/bundles`  
Cargo package: `amigo-runtime-bundles`  
Layer: runtime composition

## Role

Composes runtime bundles, plugin registration, and extraction/backend bridges.

## Owns

- bundle registration
- plugin assembly
- backend bridge wiring
- preset composition

## Does not own

- legacy/v2 parallel paths
- large formatting-only diffs
- silent fallback behavior
- domain guessing outside owner
- owning domain rules
- duplicating extractor logic
- app-specific names in generic bundle code

## Important files found in snapshot

- `crates/runtime/bundles/src/audio.rs`
- `crates/runtime/bundles/src/core.rs`
- `crates/runtime/bundles/src/devtools.rs`
- `crates/runtime/bundles/src/full.rs`
- `crates/runtime/bundles/src/host_viewport.rs`
- `crates/runtime/bundles/src/legacy_reexports.rs`
- `crates/runtime/bundles/src/lib.rs`
- `crates/runtime/bundles/src/platform.rs`
- `crates/runtime/bundles/src/plugin_composition.rs`
- `crates/runtime/bundles/src/render_diagnostics.rs`
- `crates/runtime/bundles/src/render_extractor_bridges/composition.rs`
- `crates/runtime/bundles/src/render_extractor_bridges/context.rs`
- `crates/runtime/bundles/src/render_extractor_bridges/host_overlay.rs`
- `crates/runtime/bundles/src/render_extractor_bridges/light_sources_2d/beacon.rs`
- `crates/runtime/bundles/src/render_extractor_bridges/light_sources_2d/camera_optical.rs`
- `crates/runtime/bundles/src/render_extractor_bridges/light_sources_2d/format.rs`

## Dependencies seen in Cargo.toml

- `amigo-2d-composition`
- `amigo-2d-physics`
- `amigo-2d-spatial`
- `amigo-3d-material`
- `amigo-3d-mesh`
- `amigo-3d-text`
- `amigo-assets`
- `amigo-audio-api`
- `amigo-audio-generated`
- `amigo-audio-mixer`
- `amigo-audio-output`
- `amigo-beacon-light-2d-plugin`
- `amigo-behavior`
- `amigo-camera-core-plugin`
- `amigo-camera-optics-plugin`
- `amigo-composite-plugin`
- `amigo-core`
- `amigo-devtools`
- `amigo-editor-ingame`
- `amigo-event-pipeline`
- `amigo-file-watch-notify`
- `amigo-focus-depth-plugin`
- `amigo-hot-reload`
- `amigo-input-actions`
- `amigo-input-api`
- `amigo-input-winit`
- `amigo-layered-image-2d-plugin`
- `amigo-light-2d-plugin`
- `amigo-material-2d-plugin`
- `amigo-math`
- `amigo-modding`
- `amigo-overlay-api`
- `amigo-particles-2d-plugin`
- `amigo-plugin-api`
- `amigo-render-api`
- `amigo-render-wgpu`
- `amigo-runtime`
- `amigo-scene`
- `amigo-scripting-api`
- `amigo-scripting-rhai`

## Documentation status

README present: `false`

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
cargo check -p amigo-runtime-bundles
cargo test -p amigo-runtime-bundles --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/runtime/bundles
rg -n "pub struct|pub enum|pub trait|impl " crates/runtime/bundles/src
```
