# platform-input-api

Path: `crates/platform/input-api`  
Cargo package: `amigo-input-api`  
Layer: platform adapter

## Role

Platform abstractions and concrete winit/notify/input implementations.

## Owns

- adapter implementation
- platform interface contracts
- event translation

## Does not own

- legacy/v2 parallel paths
- large formatting-only diffs
- silent fallback behavior
- domain guessing outside owner

## Important files found in snapshot

- `crates/platform/input-api/src/lib.rs`
- `crates/platform/input-api/README.md`

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
cargo check -p amigo-input-api
cargo test -p amigo-input-api --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/platform/input-api
rg -n "pub struct|pub enum|pub trait|impl " crates/platform/input-api/src
```
