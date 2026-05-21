# lighting-light-groups-2d

Path: `plugins/lighting/light-groups-2d`  
Cargo package: `amigo-light-groups-2d-plugin`  
Plugin id: `amigo.lighting.light-groups-2d`  
Family: `lighting`  
Kind: `semantic-source`  
Renderable: `False`  
Render participation: `none`

## Role

lighting plugin `light-groups-2d`. Owns grouped lighting authoring, roles, render contributions, and camera/bloom source participation.

## Manifest capabilities

- provides: `lighting.light_group.2d@1`
- requires: `lighting.light.2d@1`

## Manifest slots

- implements: none declared
- requires: none declared
- replaces: none declared

## Manifest targets

- reads: `LightMap`
- writes: none declared
- contributes: `SceneHighlight`
- contributes: `SceneEmissive`

## Manifest contributions

- emits: `{"domain": "camera.optics", "type": "CameraOpticsContribution2d", "policy": "ExplicitOnly"}`
- consumes: none declared

## Manifest diagnostics

- channels: `light-groups-2d.contributions`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/lighting/light-groups-2d/src/api/mod.rs`
- `plugins/lighting/light-groups-2d/src/diagnostics/mod.rs`
- `plugins/lighting/light-groups-2d/src/lib.rs`
- `plugins/lighting/light-groups-2d/src/participation/adapters/camera_optics.rs`
- `plugins/lighting/light-groups-2d/src/participation/adapters/mod.rs`
- `plugins/lighting/light-groups-2d/src/participation/mod.rs`
- `plugins/lighting/light-groups-2d/src/plugin.rs`
- `plugins/lighting/light-groups-2d/src/render_wgpu/mod.rs`
- `plugins/lighting/light-groups-2d/src/runtime/mod.rs`
- `plugins/lighting/light-groups-2d/src/scene/mod.rs`
- `plugins/lighting/light-groups-2d/src/scripting/mod.rs`
- `plugins/lighting/light-groups-2d/README.md`
- `plugins/lighting/light-groups-2d/plugin.toml`
- `plugins/lighting/light-groups-2d/docs/pipeline.md`
- `plugins/lighting/light-groups-2d/docs/contributions.md`
- `plugins/lighting/light-groups-2d/docs/diagnostics.md`
- `plugins/lighting/light-groups-2d/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-camera-optics-plugin`
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
cargo check -p amigo-light-groups-2d-plugin
cargo test -p amigo-light-groups-2d-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/lighting/light-groups-2d
rg -n "pub struct|pub enum|pub trait|impl " plugins/lighting/light-groups-2d/src
```
