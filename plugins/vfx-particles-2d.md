# vfx-particles-2d

Path: `plugins/vfx/particles-2d`  
Cargo package: `amigo-particles-2d-plugin`  
Plugin id: `amigo.vfx.particles-2d`  
Family: `vfx`  
Kind: `renderable-source`  
Renderable: `True`  
Render participation: `source-renderer`

## Role

vfx plugin `particles-2d`. Owns visual effect domain data and render contribution/candidate emission.

## Manifest capabilities

- provides: `vfx.particle.2d@1`
- requires: none declared

## Manifest slots

- implements: none declared
- requires: none declared
- replaces: none declared

## Manifest targets

- reads: none declared
- writes: `SceneColor`
- writes: `SceneAlpha`
- contributes: `SceneVelocity`
- contributes: `SceneHighlight`
- contributes: `SceneEmissive`

## Manifest contributions

- emits: `{"domain": "camera.optics", "type": "CameraOpticsContribution2d", "policy": "ExplicitOnly"}`
- emits: `{"domain": "camera.shutter_motion", "type": "MotionShutterContribution2d", "policy": "DisabledByDefault"}`
- consumes: none declared

## Manifest diagnostics

- channels: `particles-2d.render`
- channels: `particles-2d.contributions`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/vfx/particles-2d/src/api/mod.rs`
- `plugins/vfx/particles-2d/src/dev_console.rs`
- `plugins/vfx/particles-2d/src/devtools_console.rs`
- `plugins/vfx/particles-2d/src/diagnostics/mod.rs`
- `plugins/vfx/particles-2d/src/editor_provider.rs`
- `plugins/vfx/particles-2d/src/lib.rs`
- `plugins/vfx/particles-2d/src/model.rs`
- `plugins/vfx/particles-2d/src/participation/adapters/camera_optics.rs`
- `plugins/vfx/particles-2d/src/participation/adapters/mod.rs`
- `plugins/vfx/particles-2d/src/participation/adapters/shutter_motion.rs`
- `plugins/vfx/particles-2d/src/participation/mod.rs`
- `plugins/vfx/particles-2d/src/plugin.rs`
- `plugins/vfx/particles-2d/src/render_extraction.rs`
- `plugins/vfx/particles-2d/src/render_wgpu/mod.rs`
- `plugins/vfx/particles-2d/src/reset.rs`
- `plugins/vfx/particles-2d/src/runtime/mod.rs`
- `plugins/vfx/particles-2d/README.md`
- `plugins/vfx/particles-2d/plugin.toml`
- `plugins/vfx/particles-2d/docs/pipeline.md`
- `plugins/vfx/particles-2d/docs/contributions.md`
- `plugins/vfx/particles-2d/docs/diagnostics.md`
- `plugins/vfx/particles-2d/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-camera-optics-plugin`
- `amigo-capabilities`
- `amigo-core`
- `amigo-devtools`
- `amigo-editor-api`
- `amigo-editor-ingame`
- `amigo-fx`
- `amigo-light-2d-plugin`
- `amigo-math`
- `amigo-plugin-api`
- `amigo-runtime`
- `amigo-runtime-control`
- `amigo-scene`
- `amigo-session`
- `amigo-shutter-motion-plugin`
- `serde_yaml`

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
cargo check -p amigo-particles-2d-plugin
cargo test -p amigo-particles-2d-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/vfx/particles-2d
rg -n "pub struct|pub enum|pub trait|impl " plugins/vfx/particles-2d/src
```
