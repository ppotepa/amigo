# engine-state

Path: `crates/engine/state`  
Cargo package: `amigo-state`  
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

- `crates/engine/state/src/lib.rs`
- `crates/engine/state/src/state/model.rs`
- `crates/engine/state/src/state/plugin.rs`
- `crates/engine/state/src/state/reset.rs`
- `crates/engine/state/src/state/scene_service.rs`
- `crates/engine/state/src/state/session_service.rs`
- `crates/engine/state/src/state/tests.rs`
- `crates/engine/state/src/state/timers.rs`
- `crates/engine/state/README.md`

## Dependencies seen in Cargo.toml

- `amigo-core`
- `amigo-runtime`
- `amigo-scene`

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
cargo check -p amigo-state
cargo test -p amigo-state --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/state
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/state/src
```
