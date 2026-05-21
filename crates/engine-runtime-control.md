# engine-runtime-control

Path: `crates/engine/runtime-control`  
Cargo package: `amigo-runtime-control`  
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

- `crates/engine/runtime-control/src/completion.rs`
- `crates/engine/runtime-control/src/error.rs`
- `crates/engine/runtime-control/src/graph.rs`
- `crates/engine/runtime-control/src/lib.rs`
- `crates/engine/runtime-control/src/path.rs`
- `crates/engine/runtime-control/src/provider.rs`
- `crates/engine/runtime-control/src/registry.rs`
- `crates/engine/runtime-control/src/service.rs`
- `crates/engine/runtime-control/src/value.rs`

## Dependencies seen in Cargo.toml

- `amigo-core`
- `amigo-runtime`
- `amigo-scene`
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
cargo check -p amigo-runtime-control
cargo test -p amigo-runtime-control --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/runtime-control
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/runtime-control/src
```
