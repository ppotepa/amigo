# engine-editor-ingame

Path: `crates/engine/editor-ingame`  
Cargo package: `amigo-editor-ingame`  
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

- `crates/engine/editor-ingame/src/bounds.rs`
- `crates/engine/editor-ingame/src/commands.rs`
- `crates/engine/editor-ingame/src/component_registry.rs`
- `crates/engine/editor-ingame/src/input.rs`
- `crates/engine/editor-ingame/src/inspect.rs`
- `crates/engine/editor-ingame/src/layout.rs`
- `crates/engine/editor-ingame/src/lib.rs`
- `crates/engine/editor-ingame/src/overlay.rs`
- `crates/engine/editor-ingame/src/plugin.rs`
- `crates/engine/editor-ingame/src/properties.rs`
- `crates/engine/editor-ingame/src/provider_registry.rs`
- `crates/engine/editor-ingame/src/runtime_apply.rs`
- `crates/engine/editor-ingame/src/selection.rs`
- `crates/engine/editor-ingame/src/state.rs`
- `crates/engine/editor-ingame/src/tests.rs`
- `crates/engine/editor-ingame/src/theme.rs`

## Dependencies seen in Cargo.toml

- `amigo-2d-composition`
- `amigo-assets`
- `amigo-core`
- `amigo-devtools`
- `amigo-editor-api`
- `amigo-editor-authoring`
- `amigo-input-api`
- `amigo-math`
- `amigo-overlay-api`
- `amigo-runtime`
- `amigo-scene`
- `amigo-scripting-api`
- `amigo-ui`
- `amigo-ui-layout`
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
cargo check -p amigo-editor-ingame
cargo test -p amigo-editor-ingame --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/editor-ingame
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/editor-ingame/src
```
