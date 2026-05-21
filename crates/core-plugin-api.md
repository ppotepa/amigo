# core-plugin-api

Path: `crates/core/plugin-api`  
Cargo package: `amigo-plugin-api`  
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

- `crates/core/plugin-api/src/candidate.rs`
- `crates/core/plugin-api/src/capability.rs`
- `crates/core/plugin-api/src/catalog.rs`
- `crates/core/plugin-api/src/contribution.rs`
- `crates/core/plugin-api/src/diagnostics.rs`
- `crates/core/plugin-api/src/ids.rs`
- `crates/core/plugin-api/src/kinds.rs`
- `crates/core/plugin-api/src/lib.rs`
- `crates/core/plugin-api/src/manifest.rs`
- `crates/core/plugin-api/src/render_contributions.rs`
- `crates/core/plugin-api/src/scene.rs`
- `crates/core/plugin-api/src/slot.rs`
- `crates/core/plugin-api/src/status.rs`
- `crates/core/plugin-api/src/target.rs`
- `crates/core/plugin-api/src/validation.rs`

## Dependencies seen in Cargo.toml

- No direct dependencies parsed.

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
cargo check -p amigo-plugin-api
cargo test -p amigo-plugin-api --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/core/plugin-api
rg -n "pub struct|pub enum|pub trait|impl " crates/core/plugin-api/src
```
