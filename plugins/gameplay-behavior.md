# gameplay-behavior

Path: `plugins/gameplay/behavior`  
Cargo package: `amigo-behavior`  
Plugin id: `(missing)`  
Family: `gameplay`  
Kind: `(missing)`  
Renderable: `(missing)`  
Render participation: `(missing)`

## Role

gameplay plugin `behavior`. Confirm detailed ownership in plugin.toml, README, and local docs.

## Manifest capabilities

- provides: none declared
- requires: none declared

## Manifest slots

- implements: none declared
- requires: none declared
- replaces: none declared

## Manifest targets

- reads: none declared
- writes: none declared
- contributes: none declared

## Manifest contributions

- emits: none declared
- consumes: none declared

## Manifest diagnostics

- channels: none declared

## Declared docs/tests

Docs: `{}`  
Tests: `{}`

## Important files found in snapshot

- `plugins/gameplay/behavior/src/behavior/model.rs`
- `plugins/gameplay/behavior/src/behavior/plugin.rs`
- `plugins/gameplay/behavior/src/behavior/reset.rs`
- `plugins/gameplay/behavior/src/behavior/service.rs`
- `plugins/gameplay/behavior/src/behavior/tests.rs`
- `plugins/gameplay/behavior/src/lib.rs`
- `plugins/gameplay/behavior/src/runtime_capabilities.rs`
- `plugins/gameplay/behavior/src/scene_command.rs`
- `plugins/gameplay/behavior/src/systems/actions.rs`
- `plugins/gameplay/behavior/src/systems/menu.rs`
- `plugins/gameplay/behavior/src/systems/particle_profile.rs`
- `plugins/gameplay/behavior/src/systems/tests.rs`
- `plugins/gameplay/behavior/src/systems/tick.rs`
- `plugins/gameplay/behavior/src/systems.rs`
- `plugins/gameplay/behavior/README.md`

## Dependencies seen in Cargo.toml

- `amigo-2d-physics`
- `amigo-audio-api`
- `amigo-camera-core-plugin`
- `amigo-core`
- `amigo-fx`
- `amigo-input-actions`
- `amigo-input-api`
- `amigo-math`
- `amigo-particles-2d-plugin`
- `amigo-runtime`
- `amigo-scene`
- `amigo-scripting-api`
- `amigo-session`
- `amigo-shutter-motion-plugin`
- `amigo-state`
- `amigo-ui`

## Allowed changes

```text
plugin-owned domain models
plugin manifest capabilities/slots/targets/contributions
diagnostics declared by the plugin
waterfall tests for plugin-owned behavior
local docs when the plugin is touched
```

## Forbidden changes

```text
direct renderer hacks outside declared backend adapter path
app-side wiring for plugin behavior
silent fallback if a contribution is missing
legacy/v2 duplicate plugin paths
```

## Validation commands

```powershell
cargo check -p amigo-behavior
cargo test -p amigo-behavior
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/gameplay/behavior
rg -n "pub struct|pub enum|pub trait|impl " plugins/gameplay/behavior/src
```
