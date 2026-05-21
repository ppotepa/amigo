# gfx-sprite-2d

Path: `plugins/gfx/sprite-2d`  
Cargo package: `amigo-sprite-2d-plugin`  
Plugin id: `amigo.gfx.sprite-2d`  
Family: `gfx`  
Kind: `renderable-source`  
Renderable: `True`  
Render participation: `source-renderer`

## Role

gfx plugin `sprite-2d`. Owns graphics component domain data and render contribution/candidate emission for its component type.

## Manifest capabilities

- provides: `gfx.sprite.2d@1`
- requires: none declared

## Manifest slots

- implements: none declared
- requires: none declared
- replaces: none declared

## Manifest targets

- reads: none declared
- writes: `SceneColor`
- writes: `SceneAlpha`
- contributes: `SceneDepth`
- contributes: `SceneHighlight`
- contributes: `SceneVelocity`

## Manifest contributions

- emits: `{"domain": "camera.optics", "type": "CameraOpticsContribution2d", "policy": "ExplicitOnly"}`
- emits: `{"domain": "camera.focus_depth", "type": "FocusDepthContribution2d", "policy": "DerivedAtHydration"}`
- emits: `{"domain": "camera.shutter_motion", "type": "MotionShutterContribution2d", "policy": "DisabledByDefault"}`
- consumes: none declared

## Manifest diagnostics

- channels: `sprite-2d.render`
- channels: `sprite-2d.contributions`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/gfx/sprite-2d/src/api/candidate.rs`
- `plugins/gfx/sprite-2d/src/api/coverage.rs`
- `plugins/gfx/sprite-2d/src/api/mod.rs`
- `plugins/gfx/sprite-2d/src/api/response.rs`
- `plugins/gfx/sprite-2d/src/api/targets.rs`
- `plugins/gfx/sprite-2d/src/diagnostics/format.rs`
- `plugins/gfx/sprite-2d/src/diagnostics/mod.rs`
- `plugins/gfx/sprite-2d/src/lib.rs`
- `plugins/gfx/sprite-2d/src/participation/adapters/camera_optics.rs`
- `plugins/gfx/sprite-2d/src/participation/adapters/focus_depth.rs`
- `plugins/gfx/sprite-2d/src/participation/adapters/mod.rs`
- `plugins/gfx/sprite-2d/src/participation/adapters/shutter_motion.rs`
- `plugins/gfx/sprite-2d/src/participation/mod.rs`
- `plugins/gfx/sprite-2d/src/plugin.rs`
- `plugins/gfx/sprite-2d/src/render_wgpu/mod.rs`
- `plugins/gfx/sprite-2d/src/render_wgpu/pass.rs`
- `plugins/gfx/sprite-2d/README.md`
- `plugins/gfx/sprite-2d/plugin.toml`
- `plugins/gfx/sprite-2d/docs/pipeline.md`
- `plugins/gfx/sprite-2d/docs/contributions.md`
- `plugins/gfx/sprite-2d/docs/diagnostics.md`
- `plugins/gfx/sprite-2d/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-assets`
- `amigo-camera-optics-plugin`
- `amigo-capabilities`
- `amigo-core`
- `amigo-editor-api`
- `amigo-material-2d-plugin`
- `amigo-math`
- `amigo-plugin-api`
- `amigo-render-api`
- `amigo-runtime`
- `amigo-scene`
- `amigo-scripting-api`
- `amigo-session`

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
cargo check -p amigo-sprite-2d-plugin
cargo test -p amigo-sprite-2d-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/gfx/sprite-2d
rg -n "pub struct|pub enum|pub trait|impl " plugins/gfx/sprite-2d/src
```
