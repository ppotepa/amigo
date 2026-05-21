# ui-layout

Path: `crates/ui/layout`  
Cargo package: `amigo-ui-layout`  
Layer: UI subsystem

## Role

UI core/layout primitives and runtime-facing UI contracts.

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

- `crates/ui/layout/src/compute.rs`
- `crates/ui/layout/src/flow.rs`
- `crates/ui/layout/src/hit_test.rs`
- `crates/ui/layout/src/lib.rs`
- `crates/ui/layout/src/measure.rs`
- `crates/ui/layout/src/model.rs`
- `crates/ui/layout/src/tests.rs`
- `crates/ui/layout/src/viewport.rs`

## Dependencies seen in Cargo.toml

- `amigo-math`

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
cargo check -p amigo-ui-layout
cargo test -p amigo-ui-layout --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/ui/layout
rg -n "pub struct|pub enum|pub trait|impl " crates/ui/layout/src
```
