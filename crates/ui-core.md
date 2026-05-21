# ui-core

Path: `crates/ui/core`  
Cargo package: `amigo-ui`  
Layer: UI subsystem

## Role

UI core/layout primitives and runtime-facing UI contracts.

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

- `crates/ui/core/src/editor_capability.rs`
- `crates/ui/core/src/input.rs`
- `crates/ui/core/src/layout/adapter.rs`
- `crates/ui/core/src/layout/service.rs`
- `crates/ui/core/src/layout/tests.rs`
- `crates/ui/core/src/layout.rs`
- `crates/ui/core/src/lib.rs`
- `crates/ui/core/src/model/curve.rs`
- `crates/ui/core/src/model/document.rs`
- `crates/ui/core/src/model/events.rs`
- `crates/ui/core/src/model/node.rs`
- `crates/ui/core/src/model/style.rs`
- `crates/ui/core/src/model/theme.rs`
- `crates/ui/core/src/model.rs`
- `crates/ui/core/src/plugin.rs`
- `crates/ui/core/src/reset.rs`
- `crates/ui/core/README.md`

## Dependencies seen in Cargo.toml

- `amigo-assets`
- `amigo-capabilities`
- `amigo-core`
- `amigo-editor-api`
- `amigo-math`
- `amigo-overlay-api`
- `amigo-runtime`
- `amigo-scene`
- `amigo-scripting-api`
- `amigo-session`
- `amigo-state`
- `amigo-ui-layout`

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
cargo check -p amigo-ui
cargo test -p amigo-ui --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/ui/core
rg -n "pub struct|pub enum|pub trait|impl " crates/ui/core/src
```
