# engine-input-actions

Path: `crates/engine/input-actions`  
Cargo package: `amigo-input-actions`  
Layer: crate

## Role

Project crate. Confirm exact ownership using Cargo.toml, README, and codemap before modifying.

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

- `crates/engine/input-actions/src/lib.rs`
- `crates/engine/input-actions/src/runtime_capabilities.rs`
- `crates/engine/input-actions/src/scene_command.rs`
- `crates/engine/input-actions/src/tests.rs`
- `crates/engine/input-actions/README.md`

## Dependencies seen in Cargo.toml

- `amigo-core`
- `amigo-input-api`
- `amigo-runtime`
- `amigo-scene`
- `amigo-session`

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
cargo check -p amigo-input-actions
cargo test -p amigo-input-actions --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/input-actions
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/input-actions/src
```
