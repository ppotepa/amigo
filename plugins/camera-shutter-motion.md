# camera-shutter-motion

Path: `plugins/camera/shutter-motion`  
Cargo package: `amigo-shutter-motion-plugin`  
Plugin id: `amigo.camera.shutter-motion`  
Family: `camera`  
Kind: `target-consumer`  
Renderable: `False`  
Render participation: `target-consumer`

## Role

camera plugin `shutter-motion`. Owns shutter/motion camera semantics and motion blur contribution path.

## Manifest capabilities

- provides: `camera.shutter_motion.2d@1`
- requires: `camera.frame_context.2d@1`

## Manifest slots

- implements: `camera.shutter_model.2d`
- requires: `camera.frame_provider.2d`
- replaces: none declared

## Manifest targets

- reads: `SceneVelocity`
- reads: `SceneColor`
- writes: `SceneColor`
- contributes: none declared

## Manifest contributions

- emits: none declared
- consumes: `{"domain": "camera.shutter_motion", "type": "MotionShutterContribution2d", "policy": "DisabledByDefault"}`

## Manifest diagnostics

- channels: `shutter-motion.candidates`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/camera/shutter-motion/src/api/candidate.rs`
- `plugins/camera/shutter-motion/src/api/coverage.rs`
- `plugins/camera/shutter-motion/src/api/mod.rs`
- `plugins/camera/shutter-motion/src/api/response.rs`
- `plugins/camera/shutter-motion/src/api/source.rs`
- `plugins/camera/shutter-motion/src/api/targets.rs`
- `plugins/camera/shutter-motion/src/diagnostics/format.rs`
- `plugins/camera/shutter-motion/src/diagnostics/mod.rs`
- `plugins/camera/shutter-motion/src/lib.rs`
- `plugins/camera/shutter-motion/src/manifest.rs`
- `plugins/camera/shutter-motion/src/motion/bounds.rs`
- `plugins/camera/shutter-motion/src/motion/controller.rs`
- `plugins/camera/shutter-motion/src/motion/freeflight.rs`
- `plugins/camera/shutter-motion/src/motion/math.rs`
- `plugins/camera/shutter-motion/src/motion/mod.rs`
- `plugins/camera/shutter-motion/src/motion/plugin.rs`
- `plugins/camera/shutter-motion/README.md`
- `plugins/camera/shutter-motion/plugin.toml`
- `plugins/camera/shutter-motion/docs/pipeline.md`
- `plugins/camera/shutter-motion/docs/contributions.md`
- `plugins/camera/shutter-motion/docs/diagnostics.md`
- `plugins/camera/shutter-motion/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-2d-physics`
- `amigo-capabilities`
- `amigo-core`
- `amigo-input-actions`
- `amigo-input-api`
- `amigo-math`
- `amigo-plugin-api`
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
cargo check -p amigo-shutter-motion-plugin
cargo test -p amigo-shutter-motion-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/camera/shutter-motion
rg -n "pub struct|pub enum|pub trait|impl " plugins/camera/shutter-motion/src
```
