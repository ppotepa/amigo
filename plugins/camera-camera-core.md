# camera-camera-core

Path: `plugins/camera/camera-core`  
Cargo package: `amigo-camera-core-plugin`  
Plugin id: `amigo.camera.camera-core`  
Family: `camera`  
Kind: `bundle`  
Renderable: `False`  
Render participation: `none`

## Role

camera plugin `camera-core`. Confirm detailed ownership in plugin.toml, README, and local docs.

## Manifest capabilities

- provides: `camera.frame_context.2d@1`
- requires: none declared

## Manifest slots

- implements: `camera.frame_provider.2d`
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

- channels: `camera-core.binding`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/camera/camera-core/src/api/capabilities.rs`
- `plugins/camera/camera-core/src/api/capture.rs`
- `plugins/camera/camera-core/src/api/frame_context.rs`
- `plugins/camera/camera-core/src/api/mod.rs`
- `plugins/camera/camera-core/src/api/model.rs`
- `plugins/camera/camera-core/src/api/motion.rs`
- `plugins/camera/camera-core/src/api/projection.rs`
- `plugins/camera/camera-core/src/api/viewport.rs`
- `plugins/camera/camera-core/src/diagnostics/editor_capability.rs`
- `plugins/camera/camera-core/src/diagnostics/mod.rs`
- `plugins/camera/camera-core/src/lib.rs`
- `plugins/camera/camera-core/src/manifest.rs`
- `plugins/camera/camera-core/src/participation/mod.rs`
- `plugins/camera/camera-core/src/plugin.rs`
- `plugins/camera/camera-core/src/render_wgpu/mod.rs`
- `plugins/camera/camera-core/src/runtime/control.rs`
- `plugins/camera/camera-core/README.md`
- `plugins/camera/camera-core/plugin.toml`
- `plugins/camera/camera-core/docs/pipeline.md`
- `plugins/camera/camera-core/docs/contributions.md`
- `plugins/camera/camera-core/docs/diagnostics.md`
- `plugins/camera/camera-core/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-2d-spatial`
- `amigo-assets`
- `amigo-camera`
- `amigo-camera-optics-plugin`
- `amigo-camera-profiles-plugin`
- `amigo-composite-plugin`
- `amigo-core`
- `amigo-editor-api`
- `amigo-math`
- `amigo-plugin-api`
- `amigo-render-api`
- `amigo-runtime`
- `amigo-runtime-control`
- `amigo-scene`
- `amigo-session`
- `rhai`

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
cargo check -p amigo-camera-core-plugin
cargo test -p amigo-camera-core-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/camera/camera-core
rg -n "pub struct|pub enum|pub trait|impl " plugins/camera/camera-core/src
```
