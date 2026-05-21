# engine-session

Path: `crates/engine/session`  
Cargo package: `amigo-session`  
Layer: session/frame services

## Role

Frame/session lifecycle, timing, runtime session services, and frame-scoped state.

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

- `crates/engine/session/src/bootstrap.rs`
- `crates/engine/session/src/frame.rs`
- `crates/engine/session/src/frame_clock.rs`
- `crates/engine/session/src/lib.rs`
- `crates/engine/session/src/options.rs`
- `crates/engine/session/src/render_session.rs`
- `crates/engine/session/src/runtime_capabilities.rs`
- `crates/engine/session/src/runtime_session.rs`
- `crates/engine/session/src/scene_command_registry.rs`
- `crates/engine/session/src/scene_session.rs`
- `crates/engine/session/src/scheduler_session.rs`
- `crates/engine/session/src/scheduling.rs`
- `crates/engine/session/src/script_command_registry.rs`
- `crates/engine/session/src/script_session.rs`
- `crates/engine/session/src/session_runtime_capabilities.rs`
- `crates/engine/session/README.md`

## Dependencies seen in Cargo.toml

- `amigo-core`
- `amigo-runtime`
- `camino`
- `glam`

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
cargo check -p amigo-session
cargo test -p amigo-session --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/engine/session
rg -n "pub struct|pub enum|pub trait|impl " crates/engine/session/src
```
