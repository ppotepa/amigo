# engine-editor-session

Path: `crates/engine/editor-session`  
Cargo package: `amigo-editor-session`  
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

- `crates/engine/editor-session/src/document_state.rs`
- `crates/engine/editor-session/src/editor_panels.rs`
- `crates/engine/editor-session/src/editor_viewport.rs`
- `crates/engine/editor-session/src/lib.rs`
- `crates/engine/editor-session/src/preview_runtime.rs`
- `crates/engine/editor-session/src/selection.rs`
- `crates/engine/editor-session/src/session.rs`
- `crates/engine/editor-session/src/undo_redo.rs`

## Dependencies seen in Cargo.toml

- `amigo-core`
- `amigo-editor-api`
- `amigo-session`

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
cargo check -p amigo-editor-session
cargo test -p amigo-editor-session --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/editor-session
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/editor-session/src
```
