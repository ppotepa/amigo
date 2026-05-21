# vfx-trails-2d

Path: `plugins/vfx/trails-2d`  
Cargo package: `amigo-trails-2d-plugin`  
Plugin id: `amigo.vfx.trails-2d`  
Family: `vfx`  
Kind: `renderable-source`  
Renderable: `True`  
Render participation: `source-renderer`

## Role

vfx plugin `trails-2d`. Owns visual effect domain data and render contribution/candidate emission.

## Manifest capabilities

- provides: `vfx.trails.2d@1`
- requires: none declared

## Manifest slots

- implements: none declared
- requires: none declared
- replaces: none declared

## Manifest targets

- reads: none declared
- writes: `SceneColor`
- writes: `SceneAlpha`
- contributes: `SceneHighlight`
- contributes: `SceneEmissive`
- contributes: `SceneVelocity`

## Manifest contributions

- emits: `{"domain": "camera.optics", "type": "CameraOpticsContribution2d", "policy": "ExplicitOnly"}`
- emits: `{"domain": "camera.shutter_motion", "type": "MotionShutterContribution2d", "policy": "DisabledByDefault"}`
- consumes: none declared

## Manifest diagnostics

- channels: `trails-2d.render`
- channels: `trails-2d.contributions`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/vfx/trails-2d/src/api/mod.rs`
- `plugins/vfx/trails-2d/src/diagnostics/mod.rs`
- `plugins/vfx/trails-2d/src/lib.rs`
- `plugins/vfx/trails-2d/src/participation/adapters/mod.rs`
- `plugins/vfx/trails-2d/src/participation/adapters/shutter_motion.rs`
- `plugins/vfx/trails-2d/src/participation/mod.rs`
- `plugins/vfx/trails-2d/src/plugin.rs`
- `plugins/vfx/trails-2d/src/render_wgpu/mod.rs`
- `plugins/vfx/trails-2d/src/runtime/mod.rs`
- `plugins/vfx/trails-2d/src/scene/mod.rs`
- `plugins/vfx/trails-2d/src/scripting/mod.rs`
- `plugins/vfx/trails-2d/README.md`
- `plugins/vfx/trails-2d/plugin.toml`
- `plugins/vfx/trails-2d/docs/pipeline.md`
- `plugins/vfx/trails-2d/docs/contributions.md`
- `plugins/vfx/trails-2d/docs/diagnostics.md`
- `plugins/vfx/trails-2d/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-plugin-api`
- `amigo-shutter-motion-plugin`

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
cargo check -p amigo-trails-2d-plugin
cargo test -p amigo-trails-2d-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/vfx/trails-2d
rg -n "pub struct|pub enum|pub trait|impl " plugins/vfx/trails-2d/src
```
