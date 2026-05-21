# engine-devtools

Path: `crates/engine/devtools`  
Cargo package: `amigo-devtools`  
Layer: developer tooling runtime

## Role

Console, diagnostics, overlay, debug commands, and in-runtime developer surfaces.

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

- `crates/engine/devtools/src/builder.rs`
- `crates/engine/devtools/src/capabilities.rs`
- `crates/engine/devtools/src/command_runtime.rs`
- `crates/engine/devtools/src/commands/assets.rs`
- `crates/engine/devtools/src/commands/clock.rs`
- `crates/engine/devtools/src/commands/composition.rs`
- `crates/engine/devtools/src/commands/core.rs`
- `crates/engine/devtools/src/commands/debug/audio.rs`
- `crates/engine/devtools/src/commands/debug/dump.rs`
- `crates/engine/devtools/src/commands/debug/fps.rs`
- `crates/engine/devtools/src/commands/debug/fps_graph.rs`
- `crates/engine/devtools/src/commands/debug/graphs.rs`
- `crates/engine/devtools/src/commands/debug/input.rs`
- `crates/engine/devtools/src/commands/debug/layers.rs`
- `crates/engine/devtools/src/commands/debug/lights.rs`
- `crates/engine/devtools/src/commands/debug/memory.rs`

## Dependencies seen in Cargo.toml

- `amigo-2d-composition`
- `amigo-assets`
- `amigo-audio-output`
- `amigo-core`
- `amigo-editor-api`
- `amigo-input-api`
- `amigo-math`
- `amigo-overlay-api`
- `amigo-plugin-api`
- `amigo-render-api`
- `amigo-runtime`
- `amigo-runtime-control`
- `amigo-scene`
- `amigo-scripting-api`
- `amigo-session`

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
cargo check -p amigo-devtools
cargo test -p amigo-devtools --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/devtools
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/devtools/src
```
