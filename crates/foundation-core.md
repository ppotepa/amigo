# foundation-core

Path: `crates/foundation/core`  
Cargo package: `amigo-core`  
Layer: foundation utility

## Role

Low-level reusable utility/math/fx primitives. Keep dependency direction downward only.

## Owns

- pure utilities
- small reusable primitives
- dependency-light helpers

## Does not own

- legacy/v2 parallel paths
- large formatting-only diffs
- silent fallback behavior
- domain guessing outside owner

## Important files found in snapshot

- `crates/foundation/core/src/lib.rs`
- `crates/foundation/core/README.md`

## Dependencies seen in Cargo.toml

- No direct dependencies parsed.

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
cargo check -p amigo-core
cargo test -p amigo-core --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/foundation/core
rg -n "pub struct|pub enum|pub trait|impl " crates/foundation/core/src
```
