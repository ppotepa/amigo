# 3d-text

Path: `crates/3d/text`  
Cargo package: `amigo-3d-text`  
Layer: 3D domain support

## Role

3D mesh/text/material support. Keep independent from 2D renderer hacks.

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

- `crates/3d/text/src/editor_capability.rs`
- `crates/3d/text/src/lib.rs`
- `crates/3d/text/src/render_extraction.rs`
- `crates/3d/text/src/reset.rs`
- `crates/3d/text/src/runtime_capabilities.rs`
- `crates/3d/text/src/scene_command.rs`
- `crates/3d/text/src/script_command.rs`
- `crates/3d/text/README.md`

## Dependencies seen in Cargo.toml

- `amigo-assets`
- `amigo-capabilities`
- `amigo-core`
- `amigo-editor-api`
- `amigo-math`
- `amigo-runtime`
- `amigo-scene`
- `amigo-scripting-api`
- `amigo-session`

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
cargo check -p amigo-3d-text
cargo test -p amigo-3d-text --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/3d/text
rg -n "pub struct|pub enum|pub trait|impl " crates/3d/text/src
```
