# core-codemap-api

Path: `crates/core/codemap-api`  
Cargo package: `amigo-codemap-api`  
Layer: codemap API

## Role

Codemap/navigation contracts used by tools and agents.

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

- `crates/core/codemap-api/src/edge.rs`
- `crates/core/codemap-api/src/graph.rs`
- `crates/core/codemap-api/src/lib.rs`
- `crates/core/codemap-api/src/navigation.rs`
- `crates/core/codemap-api/src/node.rs`
- `crates/core/codemap-api/src/validation.rs`
- `crates/core/codemap-api/src/waterfall.rs`

## Dependencies seen in Cargo.toml

- `amigo-plugin-api`

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
cargo check -p amigo-codemap-api
cargo test -p amigo-codemap-api --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/core/codemap-api
rg -n "pub struct|pub enum|pub trait|impl " crates/core/codemap-api/src
```
