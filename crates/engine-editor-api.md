# engine-editor-api

Path: `crates/engine/editor-api`  
Cargo package: `amigo-editor-api`  
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

- `crates/engine/editor-api/src/asset_picker.rs`
- `crates/engine/editor-api/src/editor_capability.rs`
- `crates/engine/editor-api/src/editor_command.rs`
- `crates/engine/editor-api/src/editor_registry.rs`
- `crates/engine/editor-api/src/gizmo.rs`
- `crates/engine/editor-api/src/inspect_request.rs`
- `crates/engine/editor-api/src/inspector_model.rs`
- `crates/engine/editor-api/src/inspector_schema.rs`
- `crates/engine/editor-api/src/lib.rs`
- `crates/engine/editor-api/src/preview.rs`
- `crates/engine/editor-api/src/property_descriptor.rs`
- `crates/engine/editor-api/src/provider.rs`
- `crates/engine/editor-api/src/runtime_apply.rs`
- `crates/engine/editor-api/src/undo_redo.rs`
- `crates/engine/editor-api/src/validation.rs`

## Dependencies seen in Cargo.toml

- `amigo-core`
- `amigo-runtime`
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
cargo check -p amigo-editor-api
cargo test -p amigo-editor-api --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/editor-api
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/editor-api/src
```
