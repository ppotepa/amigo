# 2d-material-api

Path: `crates/2d/material-api`  
Cargo package: `amigo-material-api`  
Layer: 2D domain support

## Role

2D composition/material/spatial/physics support used by plugins/runtime.

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

- `crates/2d/material-api/src/lib.rs`

## Dependencies seen in Cargo.toml

- `amigo-camera`
- `serde`

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
cargo check -p amigo-material-api
cargo test -p amigo-material-api --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/2d/material-api
rg -n "pub struct|pub enum|pub trait|impl " crates/2d/material-api/src
```
