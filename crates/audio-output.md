# audio-output

Path: `crates/audio/output`  
Cargo package: `amigo-audio-output`  
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

- `crates/audio/output/src/lib.rs`
- `crates/audio/output/src/output/plugin.rs`
- `crates/audio/output/src/output/reset.rs`
- `crates/audio/output/src/output/systems.rs`
- `crates/audio/output/src/output/worker.rs`
- `crates/audio/output/src/tests.rs`
- `crates/audio/output/README.md`

## Dependencies seen in Cargo.toml

- `amigo-assets`
- `amigo-audio-api`
- `amigo-audio-mixer`
- `amigo-capabilities`
- `amigo-core`
- `amigo-runtime`
- `amigo-scene`
- `amigo-session`
- `cpal`

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
cargo check -p amigo-audio-output
cargo test -p amigo-audio-output --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/audio/output
rg -n "pub struct|pub enum|pub trait|impl " crates/audio/output/src
```
