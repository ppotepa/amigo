# 2d-physics

Path: `crates/2d/physics`  
Cargo package: `amigo-2d-physics`  
Layer: 2D domain support

## Role

2D composition/material/spatial/physics support used by plugins/runtime.

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

- `crates/2d/physics/src/events.rs`
- `crates/2d/physics/src/lib.rs`
- `crates/2d/physics/src/model.rs`
- `crates/2d/physics/src/plugin.rs`
- `crates/2d/physics/src/registry.rs`
- `crates/2d/physics/src/reset.rs`
- `crates/2d/physics/src/runtime_capabilities.rs`
- `crates/2d/physics/src/scene_command.rs`
- `crates/2d/physics/src/scene_commands/aabb_collider.rs`
- `crates/2d/physics/src/scene_commands/circle_collider.rs`
- `crates/2d/physics/src/scene_commands/collision_rules.rs`
- `crates/2d/physics/src/scene_commands/kinematic_body.rs`
- `crates/2d/physics/src/scene_commands/mod.rs`
- `crates/2d/physics/src/scene_commands/static_collider.rs`
- `crates/2d/physics/src/scene_commands/trigger.rs`
- `crates/2d/physics/src/selectors.rs`
- `crates/2d/physics/README.md`

## Dependencies seen in Cargo.toml

- `amigo-capabilities`
- `amigo-core`
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
cargo check -p amigo-2d-physics
cargo test -p amigo-2d-physics --lib
```

For docs-only edits related to this crate:

```powershell
git diff --check
```

## Navigation queries

```powershell
rg -n "TODO|FIXME|panic!|unwrap\(" crates/2d/physics
rg -n "pub struct|pub enum|pub trait|impl " crates/2d/physics/src
```
