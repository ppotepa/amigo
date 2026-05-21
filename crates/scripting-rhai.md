# scripting-rhai

Path: `crates/scripting/rhai`  
Cargo package: `amigo-scripting-rhai`  
Layer: scripting layer

## Role

Scripting contracts and Rhai backend integration.

## Owns

- local crate-owned contracts
- tests for crate-owned behavior
- small targeted fixes inside ownership boundary

## Does not own

- legacy/v2 parallel paths
- large formatting-only diffs
- silent fallback behavior
- domain guessing outside owner

## Important files found in snapshot

- `crates/scripting/rhai/src/bindings/actions.rs`
- `crates/scripting/rhai/src/bindings/arcade.rs`
- `crates/scripting/rhai/src/bindings/assets.rs`
- `crates/scripting/rhai/src/bindings/audio.rs`
- `crates/scripting/rhai/src/bindings/beacon2d.rs`
- `crates/scripting/rhai/src/bindings/camera.rs`
- `crates/scripting/rhai/src/bindings/commands.rs`
- `crates/scripting/rhai/src/bindings/common.rs`
- `crates/scripting/rhai/src/bindings/debug.rs`
- `crates/scripting/rhai/src/bindings/entities.rs`
- `crates/scripting/rhai/src/bindings/input.rs`
- `crates/scripting/rhai/src/bindings/layered_image2d.rs`
- `crates/scripting/rhai/src/bindings/light2d.rs`
- `crates/scripting/rhai/src/bindings/material3d.rs`
- `crates/scripting/rhai/src/bindings/mesh3d.rs`
- `crates/scripting/rhai/src/bindings/mod.rs`
- `crates/scripting/rhai/README.md`

## Dependencies seen in Cargo.toml

- `amigo-2d-physics`
- `amigo-assets`
- `amigo-camera-core-plugin`
- `amigo-capabilities`
- `amigo-composite-plugin`
- `amigo-core`
- `amigo-editor-api`
- `amigo-focus-depth-plugin`
- `amigo-fx`
- `amigo-input-actions`
- `amigo-input-api`
- `amigo-light-2d-plugin`
- `amigo-math`
- `amigo-modding`
- `amigo-particles-2d-plugin`
- `amigo-plugin-api`
- `amigo-render-api`
- `amigo-runtime`
- `amigo-runtime-control`
- `amigo-scene`
- `amigo-scripting-api`
- `amigo-session`
- `amigo-shutter-motion-plugin`
- `amigo-sprite-2d-plugin`
- `amigo-state`
- `amigo-ui`
- `amigo-vector-2d-plugin`
- `rhai`
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
cargo check -p amigo-scripting-rhai
cargo test -p amigo-scripting-rhai --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/scripting/rhai
rg -n "pub struct|pub enum|pub trait|impl " crates/scripting/rhai/src
```
