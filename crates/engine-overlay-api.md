# engine-overlay-api

Path: `crates/engine/overlay-api`  
Cargo package: `amigo-overlay-api`  
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

- `crates/engine/overlay-api/src/helpers.rs`
- `crates/engine/overlay-api/src/layout/adapter.rs`
- `crates/engine/overlay-api/src/layout/entry.rs`
- `crates/engine/overlay-api/src/layout/tabs.rs`
- `crates/engine/overlay-api/src/layout.rs`
- `crates/engine/overlay-api/src/lib.rs`
- `crates/engine/overlay-api/src/primitives.rs`
- `crates/engine/overlay-api/src/widgets/basic.rs`
- `crates/engine/overlay-api/src/widgets/color_curve.rs`
- `crates/engine/overlay-api/src/widgets/dropdown.rs`
- `crates/engine/overlay-api/src/widgets/tabs.rs`
- `crates/engine/overlay-api/src/widgets.rs`

## Dependencies seen in Cargo.toml

- `amigo-assets`
- `amigo-math`
- `amigo-ui-layout`

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
cargo check -p amigo-overlay-api
cargo test -p amigo-overlay-api --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/overlay-api
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/overlay-api/src
```
