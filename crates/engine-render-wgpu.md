# engine-render-wgpu

Path: `crates/engine/render-wgpu`  
Cargo package: `amigo-render-wgpu`  
Layer: WGPU backend

## Role

Concrete backend implementation for render-api contracts, pipelines, frame graph, resources, and passes.

## Owns

- pipeline/resource implementation
- render graph execution
- target allocation
- backend diagnostics

## Does not own

- legacy/v2 parallel paths
- large formatting-only diffs
- silent fallback behavior
- domain guessing outside owner
- authoring semantics
- new central PostFx match without descriptor plan
- renderer-side guesses from scene existence

## Important files found in snapshot

- `crates/engine/render-wgpu/src/backend/helpers.rs`
- `crates/engine/render-wgpu/src/backend/mod.rs`
- `crates/engine/render-wgpu/src/backend/surface.rs`
- `crates/engine/render-wgpu/src/backend/types.rs`
- `crates/engine/render-wgpu/src/frame_packet.rs`
- `crates/engine/render-wgpu/src/lib.rs`
- `crates/engine/render-wgpu/src/plugin_pass.rs`
- `crates/engine/render-wgpu/src/renderable_adapter.rs`
- `crates/engine/render-wgpu/src/renderable_adapters/beacon.rs`
- `crates/engine/render-wgpu/src/renderable_adapters/layered_image.rs`
- `crates/engine/render-wgpu/src/renderable_adapters/mod.rs`
- `crates/engine/render-wgpu/src/renderable_adapters/particle.rs`
- `crates/engine/render-wgpu/src/renderable_adapters/sprite.rs`
- `crates/engine/render-wgpu/src/renderable_adapters/text.rs`
- `crates/engine/render-wgpu/src/renderable_adapters/tilemap.rs`
- `crates/engine/render-wgpu/src/renderable_adapters/vector.rs`
- `crates/engine/render-wgpu/README.md`

## Dependencies seen in Cargo.toml

- `amigo-2d-composition`
- `amigo-2d-spatial`
- `amigo-3d-material`
- `amigo-3d-mesh`
- `amigo-3d-text`
- `amigo-assets`
- `amigo-beacon-light-2d-plugin`
- `amigo-camera-optics-plugin`
- `amigo-composite-plugin`
- `amigo-core`
- `amigo-focus-depth-plugin`
- `amigo-font`
- `amigo-fx`
- `amigo-layered-image-2d-plugin`
- `amigo-light-2d-plugin`
- `amigo-material-2d-plugin`
- `amigo-math`
- `amigo-overlay-api`
- `amigo-particles-2d-plugin`
- `amigo-plugin-api`
- `amigo-relight-2d-plugin`
- `amigo-render-api`
- `amigo-runtime`
- `amigo-scene`
- `amigo-sprite-2d-plugin`
- `amigo-text-2d-plugin`
- `amigo-tilemap-2d-plugin`
- `amigo-ui-layout`
- `amigo-vector-2d-plugin`
- `amigo-window-api`
- `fontdue`
- `image`
- `wgpu`

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
cargo check -p amigo-render-wgpu
cargo test -p amigo-render-wgpu --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/render-wgpu
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/render-wgpu/src
```
