# audio-generated

Path: `crates/audio/generated`  
Cargo package: `amigo-audio-generated`  
Layer: audio subsystem

## Role

Audio API/generated assets/mixer/output layering.

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

- `crates/audio/generated/src/lib.rs`
- `crates/audio/generated/src/parser.rs`
- `crates/audio/generated/src/plugin.rs`
- `crates/audio/generated/src/render.rs`
- `crates/audio/generated/src/tests.rs`
- `crates/audio/generated/src/types.rs`
- `crates/audio/generated/README.md`

## Dependencies seen in Cargo.toml

- `amigo-assets`
- `amigo-capabilities`
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
cargo check -p amigo-audio-generated
cargo test -p amigo-audio-generated --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/audio/generated
rg -n "pub struct|pub enum|pub trait|impl " crates/audio/generated/src
```
