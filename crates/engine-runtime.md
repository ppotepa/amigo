# engine-runtime

Path: `crates/engine/runtime`  
Cargo package: `amigo-runtime`  
Layer: runtime core

## Role

Plugin/system runtime contracts, service registration, phases, and scheduling interfaces.

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

- `crates/engine/runtime/src/bundle.rs`
- `crates/engine/runtime/src/handler_registry.rs`
- `crates/engine/runtime/src/lib.rs`
- `crates/engine/runtime/src/schedule.rs`
- `crates/engine/runtime/src/scheduling.rs`
- `crates/engine/runtime/src/task_system.rs`
- `crates/engine/runtime/README.md`

## Dependencies seen in Cargo.toml

- `amigo-core`

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
cargo check -p amigo-runtime
cargo test -p amigo-runtime --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/runtime
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/runtime/src
```
