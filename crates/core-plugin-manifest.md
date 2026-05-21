# core-plugin-manifest

Path: `crates/core/plugin-manifest`  
Cargo package: `amigo-plugin-manifest`  
Layer: plugin infrastructure

## Role

Plugin API, manifests, loader/index mechanics, and plugin metadata contracts.

## Owns

- manifest/schema handling
- plugin loading/indexing
- capability contracts

## Does not own

- legacy/v2 parallel paths
- large formatting-only diffs
- silent fallback behavior
- domain guessing outside owner

## Important files found in snapshot

- `crates/core/plugin-manifest/src/error.rs`
- `crates/core/plugin-manifest/src/lib.rs`
- `crates/core/plugin-manifest/src/parse.rs`
- `crates/core/plugin-manifest/src/raw.rs`

## Dependencies seen in Cargo.toml

- `amigo-plugin-api`
- `serde`
- `toml`

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
cargo check -p amigo-plugin-manifest
cargo test -p amigo-plugin-manifest --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/core/plugin-manifest
rg -n "pub struct|pub enum|pub trait|impl " crates/core/plugin-manifest/src
```
