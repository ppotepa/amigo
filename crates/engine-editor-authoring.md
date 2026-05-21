# engine-editor-authoring

Path: `crates/engine/editor-authoring`  
Cargo package: `amigo-editor-authoring`  
Layer: editor subsystem

## Role

Editor contracts/sessions/authoring/runtime ingame overlay depending on suffix.

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

- `crates/engine/editor-authoring/src/bindings.rs`
- `crates/engine/editor-authoring/src/graph.rs`
- `crates/engine/editor-authoring/src/ids.rs`
- `crates/engine/editor-authoring/src/image_parts.rs`
- `crates/engine/editor-authoring/src/inspect/component.rs`
- `crates/engine/editor-authoring/src/inspect/draw_layer.rs`
- `crates/engine/editor-authoring/src/inspect/mod.rs`
- `crates/engine/editor-authoring/src/inspect/post_fx.rs`
- `crates/engine/editor-authoring/src/inspect/properties.rs`
- `crates/engine/editor-authoring/src/inspect/raw.rs`
- `crates/engine/editor-authoring/src/inspect/scene_object.rs`
- `crates/engine/editor-authoring/src/inspect/visibility.rs`
- `crates/engine/editor-authoring/src/lib.rs`
- `crates/engine/editor-authoring/src/loader.rs`
- `crates/engine/editor-authoring/src/node_descriptors.rs`
- `crates/engine/editor-authoring/src/plugin.rs`

## Dependencies seen in Cargo.toml

- `amigo-core`
- `amigo-editor-api`
- `amigo-modding`
- `amigo-runtime`
- `amigo-scene`
- `amigo-session`
- `serde`
- `serde_yaml`

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
cargo check -p amigo-editor-authoring
cargo test -p amigo-editor-authoring --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/editor-authoring
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/editor-authoring/src
```
