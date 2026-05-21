# lighting-light-2d

Path: `plugins/lighting/light-2d`  
Cargo package: `amigo-light-2d-plugin`  
Plugin id: `amigo.lighting.light-2d`  
Family: `lighting`  
Kind: `semantic-source`  
Renderable: `False`  
Render participation: `none`

## Role

lighting plugin `light-2d`. Confirm detailed ownership in plugin.toml, README, and local docs.

## Manifest capabilities

- provides: `lighting.light.2d@1`
- requires: none declared

## Manifest slots

- implements: none declared
- requires: none declared
- replaces: none declared

## Manifest targets

- reads: none declared
- writes: `SceneLighting`
- contributes: `SceneHighlight`
- contributes: `SceneEmissive`

## Manifest contributions

- emits: `{"domain": "camera.optics", "type": "CameraOpticsContribution2d", "policy": "ExplicitOnly"}`
- consumes: none declared

## Manifest diagnostics

- channels: `light-2d.contributions`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/lighting/light-2d/src/api/mod.rs`
- `plugins/lighting/light-2d/src/diagnostics/mod.rs`
- `plugins/lighting/light-2d/src/lib.rs`
- `plugins/lighting/light-2d/src/lighting/dev_console.rs`
- `plugins/lighting/light-2d/src/lighting/mod.rs`
- `plugins/lighting/light-2d/src/lighting/model.rs`
- `plugins/lighting/light-2d/src/lighting/plugin.rs`
- `plugins/lighting/light-2d/src/lighting/render_extraction.rs`
- `plugins/lighting/light-2d/src/lighting/reset.rs`
- `plugins/lighting/light-2d/src/lighting/runtime_capabilities.rs`
- `plugins/lighting/light-2d/src/lighting/scene_bridge.rs`
- `plugins/lighting/light-2d/src/lighting/scene_command.rs`
- `plugins/lighting/light-2d/src/lighting/script_command.rs`
- `plugins/lighting/light-2d/src/lighting/service.rs`
- `plugins/lighting/light-2d/src/lighting/tests.rs`
- `plugins/lighting/light-2d/src/participation/adapters/camera_optics.rs`
- `plugins/lighting/light-2d/README.md`
- `plugins/lighting/light-2d/plugin.toml`
- `plugins/lighting/light-2d/docs/pipeline.md`
- `plugins/lighting/light-2d/docs/contributions.md`
- `plugins/lighting/light-2d/docs/diagnostics.md`
- `plugins/lighting/light-2d/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-assets`
- `amigo-camera-optics-plugin`
- `amigo-capabilities`
- `amigo-core`
- `amigo-devtools`
- `amigo-layered-image-2d-plugin`
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
cargo check -p amigo-light-2d-plugin
cargo test -p amigo-light-2d-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/lighting/light-2d
rg -n "pub struct|pub enum|pub trait|impl " plugins/lighting/light-2d/src
```
