# engine-camera

Path: `crates/engine/camera`  
Cargo package: `amigo-camera`  
Layer: camera contracts

## Role

Shared camera data/contracts used by domain plugins and render extraction.

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

- `crates/engine/camera/src/capture.rs`
- `crates/engine/camera/src/debug_view.rs`
- `crates/engine/camera/src/id.rs`
- `crates/engine/camera/src/lib.rs`
- `crates/engine/camera/src/optical.rs`

## Dependencies seen in Cargo.toml

- `amigo-2d-spatial`
- `amigo-core`
- `amigo-math`
- `amigo-plugin-api`
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
cargo check -p amigo-camera
cargo test -p amigo-camera --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/camera
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/camera/src
```
