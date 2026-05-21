# engine-hot-reload

Path: `crates/engine/hot-reload`  
Cargo package: `amigo-hot-reload`  
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

- `crates/engine/hot-reload/src/lib.rs`
- `crates/engine/hot-reload/src/tests.rs`
- `crates/engine/hot-reload/README.md`

## Dependencies seen in Cargo.toml

- `amigo-core`
- `amigo-runtime`

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
cargo check -p amigo-hot-reload
cargo test -p amigo-hot-reload --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/hot-reload
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/hot-reload/src
```
