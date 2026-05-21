# lighting-beacon-light-2d

Path: `plugins/lighting/beacon-light-2d`  
Cargo package: `amigo-beacon-light-2d-plugin`  
Plugin id: `amigo.lighting.beacon-light-2d`  
Family: `lighting`  
Kind: `semantic-source`  
Renderable: `True`  
Render participation: `source-renderer`

## Role

lighting plugin `beacon-light-2d`. Confirm detailed ownership in plugin.toml, README, and local docs.

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
- writes: `SceneColor`
- contributes: `SceneHighlight`
- contributes: `SceneEmissive`

## Manifest contributions

- emits: `{"domain": "camera.optics", "type": "CameraOpticsContribution2d", "policy": "ExplicitOnly"}`
- consumes: none declared

## Manifest diagnostics

- channels: `beacon-light-2d.contributions`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/lighting/beacon-light-2d/src/api/mod.rs`
- `plugins/lighting/beacon-light-2d/src/beacon/control.rs`
- `plugins/lighting/beacon-light-2d/src/beacon/editor_capability.rs`
- `plugins/lighting/beacon-light-2d/src/beacon/mod.rs`
- `plugins/lighting/beacon-light-2d/src/beacon/model.rs`
- `plugins/lighting/beacon-light-2d/src/beacon/plugin.rs`
- `plugins/lighting/beacon-light-2d/src/beacon/render_extraction.rs`
- `plugins/lighting/beacon-light-2d/src/beacon/reset.rs`
- `plugins/lighting/beacon-light-2d/src/beacon/runtime_capabilities.rs`
- `plugins/lighting/beacon-light-2d/src/beacon/scene_bridge.rs`
- `plugins/lighting/beacon-light-2d/src/beacon/scene_command.rs`
- `plugins/lighting/beacon-light-2d/src/beacon/script_command.rs`
- `plugins/lighting/beacon-light-2d/src/beacon/service.rs`
- `plugins/lighting/beacon-light-2d/src/beacon/tests.rs`
- `plugins/lighting/beacon-light-2d/src/diagnostics/mod.rs`
- `plugins/lighting/beacon-light-2d/src/lib.rs`
- `plugins/lighting/beacon-light-2d/README.md`
- `plugins/lighting/beacon-light-2d/plugin.toml`
- `plugins/lighting/beacon-light-2d/docs/pipeline.md`
- `plugins/lighting/beacon-light-2d/docs/contributions.md`
- `plugins/lighting/beacon-light-2d/docs/diagnostics.md`
- `plugins/lighting/beacon-light-2d/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-camera-optics-plugin`
- `amigo-capabilities`
- `amigo-core`
- `amigo-math`
- `amigo-plugin-api`
- `amigo-relight-2d-plugin`
- `amigo-render-api`
- `amigo-runtime`
- `amigo-runtime-control`
- `amigo-scene`
- `amigo-scripting-api`
- `amigo-session`
- `amigo-shutter-motion-plugin`
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
cargo check -p amigo-beacon-light-2d-plugin
cargo test -p amigo-beacon-light-2d-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/lighting/beacon-light-2d
rg -n "pub struct|pub enum|pub trait|impl " plugins/lighting/beacon-light-2d/src
```
