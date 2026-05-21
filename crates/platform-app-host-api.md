# platform-app-host-api

Path: `crates/platform/app-host-api`  
Cargo package: `amigo-app-host-api`  
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

- `crates/platform/app-host-api/src/lib.rs`
- `crates/platform/app-host-api/README.md`

## Dependencies seen in Cargo.toml

- `amigo-core`
- `amigo-input-api`
- `amigo-window-api`

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
cargo check -p amigo-app-host-api
cargo test -p amigo-app-host-api --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/platform/app-host-api
rg -n "pub struct|pub enum|pub trait|impl " crates/platform/app-host-api/src
```
