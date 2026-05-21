# camera-camera-profiles

Path: `plugins/camera/camera-profiles`  
Cargo package: `amigo-camera-profiles-plugin`  
Plugin id: `amigo.camera.camera-profiles`  
Family: `camera`  
Kind: `bundle`  
Renderable: `False`  
Render participation: `none`

## Role

camera plugin `camera-profiles`. Confirm detailed ownership in plugin.toml, README, and local docs.

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

- channels: `camera-profiles.catalog`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/camera/camera-profiles/src/api/mod.rs`
- `plugins/camera/camera-profiles/src/api/profile.rs`
- `plugins/camera/camera-profiles/src/api/quality.rs`
- `plugins/camera/camera-profiles/src/diagnostics/format.rs`
- `plugins/camera/camera-profiles/src/diagnostics/mod.rs`
- `plugins/camera/camera-profiles/src/lib.rs`
- `plugins/camera/camera-profiles/src/manifest.rs`
- `plugins/camera/camera-profiles/src/participation/mod.rs`
- `plugins/camera/camera-profiles/src/plugin.rs`
- `plugins/camera/camera-profiles/src/render_wgpu/mod.rs`
- `plugins/camera/camera-profiles/src/runtime/mod.rs`
- `plugins/camera/camera-profiles/src/runtime/profiles.rs`
- `plugins/camera/camera-profiles/src/runtime/registry.rs`
- `plugins/camera/camera-profiles/src/scene/mod.rs`
- `plugins/camera/camera-profiles/src/scripting/mod.rs`
- `plugins/camera/camera-profiles/README.md`
- `plugins/camera/camera-profiles/plugin.toml`
- `plugins/camera/camera-profiles/docs/pipeline.md`
- `plugins/camera/camera-profiles/docs/contributions.md`
- `plugins/camera/camera-profiles/docs/diagnostics.md`
- `plugins/camera/camera-profiles/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-assets`
- `amigo-composite-plugin`
- `amigo-film-look-plugin`
- `amigo-plugin-api`

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
cargo check -p amigo-camera-profiles-plugin
cargo test -p amigo-camera-profiles-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/camera/camera-profiles
rg -n "pub struct|pub enum|pub trait|impl " plugins/camera/camera-profiles/src
```
