# materials-material-maps

Path: `plugins/materials/material-maps`  
Cargo package: `amigo-material-maps-plugin`  
Plugin id: `amigo.materials.material-maps`  
Family: `materials`  
Kind: `target-consumer`  
Renderable: `False`  
Render participation: `target-writer`

## Role

materials plugin `material-maps`. Confirm detailed ownership in plugin.toml, README, and local docs.

## Manifest capabilities

- provides: `materials.material-maps@1`
- requires: `render.backend@1`

## Manifest slots

- implements: none declared
- requires: `render.backend`
- replaces: none declared

## Manifest targets

- reads: `SceneColor`
- writes: `SceneHighlight`
- writes: `SceneEmissive`
- writes: `SceneDepth`
- contributes: none declared

## Manifest contributions

- emits: none declared
- consumes: `{"domain": "camera.optics", "type": "CameraOpticsContribution2d", "policy": "ExplicitOnly"}`
- consumes: `{"domain": "camera.focus_depth", "type": "FocusDepthContribution2d", "policy": "DerivedAtHydration"}`

## Manifest diagnostics

- channels: `material-maps.targets`
- channels: `material-maps.diagnostics`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/materials/material-maps/src/api/mod.rs`
- `plugins/materials/material-maps/src/diagnostics/mod.rs`
- `plugins/materials/material-maps/src/lib.rs`
- `plugins/materials/material-maps/src/participation/mod.rs`
- `plugins/materials/material-maps/src/plugin.rs`
- `plugins/materials/material-maps/src/render_wgpu/mod.rs`
- `plugins/materials/material-maps/src/runtime/mod.rs`
- `plugins/materials/material-maps/src/scene/mod.rs`
- `plugins/materials/material-maps/src/scripting/mod.rs`
- `plugins/materials/material-maps/README.md`
- `plugins/materials/material-maps/plugin.toml`
- `plugins/materials/material-maps/docs/pipeline.md`
- `plugins/materials/material-maps/docs/contributions.md`
- `plugins/materials/material-maps/docs/diagnostics.md`
- `plugins/materials/material-maps/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

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
cargo check -p amigo-material-maps-plugin
cargo test -p amigo-material-maps-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/materials/material-maps
rg -n "pub struct|pub enum|pub trait|impl " plugins/materials/material-maps/src
```
