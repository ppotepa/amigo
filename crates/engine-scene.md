# engine-scene

Path: `crates/engine/scene`  
Cargo package: `amigo-scene`  
Layer: scene core

## Role

Scene document model, hydration, commands, metadata, and validation. Should not own WGPU behavior.

## Owns

- scene document/hydration/commands
- metadata provider contracts
- validation
- serialization-friendly models

## Does not own

- legacy/v2 parallel paths
- large formatting-only diffs
- silent fallback behavior
- domain guessing outside owner
- WGPU-specific fields
- renderer execution policy
- plugin-specific runtime systems when provider registration is available

## Important files found in snapshot

- `crates/engine/scene/src/command_format.rs`
- `crates/engine/scene/src/commands.rs`
- `crates/engine/scene/src/component_metadata.rs`
- `crates/engine/scene/src/component_metadata_provider.rs`
- `crates/engine/scene/src/document/behavior.rs`
- `crates/engine/scene/src/document/camera.rs`
- `crates/engine/scene/src/document/compiler.rs`
- `crates/engine/scene/src/document/components.rs`
- `crates/engine/scene/src/document/core.rs`
- `crates/engine/scene/src/document/defaults.rs`
- `crates/engine/scene/src/document/loader.rs`
- `crates/engine/scene/src/document/mod.rs`
- `crates/engine/scene/src/document/particles.rs`
- `crates/engine/scene/src/document/prefab.rs`
- `crates/engine/scene/src/document/render_contributions.rs`
- `crates/engine/scene/src/document/render_values.rs`
- `crates/engine/scene/README.md`

## Dependencies seen in Cargo.toml

- `amigo-2d-spatial`
- `amigo-assets`
- `amigo-camera`
- `amigo-core`
- `amigo-fx`
- `amigo-material-api`
- `amigo-math`
- `amigo-plugin-api`
- `amigo-render-api`
- `amigo-runtime`
- `amigo-scripting-api`
- `amigo-session`
- `serde`
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
cargo check -p amigo-scene
cargo test -p amigo-scene --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/scene
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/scene/src
```
