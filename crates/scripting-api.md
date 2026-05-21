# scripting-api

Path: `crates/scripting/api`  
Cargo package: `amigo-scripting-api`  
Layer: scripting layer

## Role

Scripting contracts and Rhai backend integration.

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

- `crates/scripting/api/src/command_handler.rs`
- `crates/scripting/api/src/dev_console_input.rs`
- `crates/scripting/api/src/lib.rs`
- `crates/scripting/api/src/runtime.rs`
- `crates/scripting/api/src/services.rs`
- `crates/scripting/api/src/tests.rs`
- `crates/scripting/api/src/types.rs`
- `crates/scripting/api/README.md`

## Dependencies seen in Cargo.toml

- `amigo-core`
- `amigo-runtime`

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
cargo check -p amigo-scripting-api
cargo test -p amigo-scripting-api --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/scripting/api
rg -n "pub struct|pub enum|pub trait|impl " crates/scripting/api/src
```
