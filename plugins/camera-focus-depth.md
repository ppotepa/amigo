# camera-focus-depth

Path: `plugins/camera/focus-depth`  
Cargo package: `amigo-focus-depth-plugin`  
Plugin id: `amigo.camera.focus-depth`  
Family: `camera`  
Kind: `target-consumer`  
Renderable: `False`  
Render participation: `target-consumer`

## Role

camera plugin `focus-depth`. Owns focus/depth camera behavior and focus blur domain integration.

## Manifest capabilities

- provides: `camera.focus_depth.2d@1`
- requires: `camera.frame_context.2d@1`

## Manifest slots

- implements: `camera.focus_model.2d`
- requires: `camera.frame_provider.2d`
- replaces: none declared

## Manifest targets

- reads: `SceneDepth`
- reads: `SceneColor`
- writes: `SceneColor`
- contributes: none declared

## Manifest contributions

- emits: none declared
- consumes: `{"domain": "camera.focus_depth", "type": "FocusDepthContribution2d", "policy": "DerivedAtHydration"}`

## Manifest diagnostics

- channels: `focus-depth.candidates`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/camera/focus-depth/src/api/candidate.rs`
- `plugins/camera/focus-depth/src/api/coverage.rs`
- `plugins/camera/focus-depth/src/api/mod.rs`
- `plugins/camera/focus-depth/src/api/response.rs`
- `plugins/camera/focus-depth/src/api/source.rs`
- `plugins/camera/focus-depth/src/api/targets.rs`
- `plugins/camera/focus-depth/src/depth_map/mod.rs`
- `plugins/camera/focus-depth/src/depth_map/model.rs`
- `plugins/camera/focus-depth/src/depth_map/plugin.rs`
- `plugins/camera/focus-depth/src/depth_map/render_extraction.rs`
- `plugins/camera/focus-depth/src/depth_map/runtime_capabilities.rs`
- `plugins/camera/focus-depth/src/depth_map/scene_bridge.rs`
- `plugins/camera/focus-depth/src/depth_map/scene_command.rs`
- `plugins/camera/focus-depth/src/depth_map/service.rs`
- `plugins/camera/focus-depth/src/diagnostics/format.rs`
- `plugins/camera/focus-depth/src/diagnostics/mod.rs`
- `plugins/camera/focus-depth/README.md`
- `plugins/camera/focus-depth/plugin.toml`
- `plugins/camera/focus-depth/docs/pipeline.md`
- `plugins/camera/focus-depth/docs/contributions.md`
- `plugins/camera/focus-depth/docs/diagnostics.md`
- `plugins/camera/focus-depth/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-2d-composition`
- `amigo-2d-spatial`
- `amigo-assets`
- `amigo-beacon-light-2d-plugin`
- `amigo-camera-core-plugin`
- `amigo-capabilities`
- `amigo-core`
- `amigo-layered-image-2d-plugin`
- `amigo-math`
- `amigo-particles-2d-plugin`
- `amigo-plugin-api`
- `amigo-render-api`
- `amigo-runtime`
- `amigo-scene`
- `amigo-session`
- `amigo-sprite-2d-plugin`
- `amigo-text-2d-plugin`
- `amigo-vector-2d-plugin`

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
cargo check -p amigo-focus-depth-plugin
cargo test -p amigo-focus-depth-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/camera/focus-depth
rg -n "pub struct|pub enum|pub trait|impl " plugins/camera/focus-depth/src
```
