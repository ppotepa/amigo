# engine-render-api

Path: `crates/engine/render-api`  
Cargo package: `amigo-render-api`  
Layer: render contract layer

## Role

Renderer-facing contracts: frame packets, graph models, targets, camera capture inputs, PostFX models.

## Owns

- contract structs/enums
- frame graph models
- render target declarations
- renderer-facing diagnostics

## Does not own

- legacy/v2 parallel paths
- large formatting-only diffs
- silent fallback behavior
- domain guessing outside owner
- WGPU implementation details
- app bootstrap knowledge
- heuristics based on component names

## Important files found in snapshot

- `crates/engine/render-api/src/camera_binding.rs`
- `crates/engine/render-api/src/camera_capture.rs`
- `crates/engine/render-api/src/composition.rs`
- `crates/engine/render-api/src/composition_layer.rs`
- `crates/engine/render-api/src/contributions.rs`
- `crates/engine/render-api/src/diagnostics.rs`
- `crates/engine/render-api/src/frame_graph.rs`
- `crates/engine/render-api/src/frame_graph_builder.rs`
- `crates/engine/render-api/src/lib.rs`
- `crates/engine/render-api/src/light_source_2d.rs`
- `crates/engine/render-api/src/post_fx.rs`
- `crates/engine/render-api/src/post_fx_document/mod.rs`
- `crates/engine/render-api/src/post_fx_document/post_fx.rs`
- `crates/engine/render-api/src/post_fx_document/post_fx_defaults.rs`
- `crates/engine/render-api/src/post_fx_document/post_fx_lens_droplets.rs`
- `crates/engine/render-api/src/post_fx_document/post_fx_rain_glass.rs`
- `crates/engine/render-api/README.md`

## Dependencies seen in Cargo.toml

- `amigo-2d-spatial`
- `amigo-camera`
- `amigo-core`
- `amigo-plugin-api`
- `amigo-session`
- `serde`

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
cargo check -p amigo-render-api
cargo test -p amigo-render-api --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/render-api
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/render-api/src
```
