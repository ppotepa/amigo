# materials-material-2d

Path: `plugins/materials/material-2d`  
Cargo package: `amigo-material-2d-plugin`  
Plugin id: `amigo.materials.material-2d`  
Family: `materials`  
Kind: `semantic-source`  
Renderable: `False`  
Render participation: `none`

## Role

materials plugin `material-2d`. Owns 2D material semantics and material-to-render/camera response mapping.

## Manifest capabilities

- provides: `materials.material.2d@1`
- requires: none declared

## Manifest slots

- implements: none declared
- requires: `scene.component_hydrator`
- replaces: none declared

## Manifest targets

- reads: none declared
- writes: none declared
- contributes: `SceneHighlight`
- contributes: `SceneEmissive`
- contributes: `SceneDepth`

## Manifest contributions

- emits: `{"domain": "camera.optics", "type": "CameraOpticsContribution2d", "policy": "ExplicitOnly"}`
- emits: `{"domain": "camera.focus_depth", "type": "FocusDepthContribution2d", "policy": "DerivedAtHydration"}`
- consumes: none declared

## Manifest diagnostics

- channels: `material-2d.contributions`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/materials/material-2d/src/api/mod.rs`
- `plugins/materials/material-2d/src/api/runtime.rs`
- `plugins/materials/material-2d/src/diagnostics/mod.rs`
- `plugins/materials/material-2d/src/lib.rs`
- `plugins/materials/material-2d/src/participation/adapters/camera_optics.rs`
- `plugins/materials/material-2d/src/participation/adapters/mod.rs`
- `plugins/materials/material-2d/src/participation/mod.rs`
- `plugins/materials/material-2d/src/plugin.rs`
- `plugins/materials/material-2d/src/render_wgpu/mod.rs`
- `plugins/materials/material-2d/src/runtime/mod.rs`
- `plugins/materials/material-2d/src/scene/document.rs`
- `plugins/materials/material-2d/src/scene/mod.rs`
- `plugins/materials/material-2d/src/scripting/mod.rs`
- `plugins/materials/material-2d/README.md`
- `plugins/materials/material-2d/plugin.toml`
- `plugins/materials/material-2d/docs/pipeline.md`
- `plugins/materials/material-2d/docs/contributions.md`
- `plugins/materials/material-2d/docs/diagnostics.md`
- `plugins/materials/material-2d/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-camera-optics-plugin`
- `amigo-material-api`
- `amigo-plugin-api`
- `serde`

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
cargo check -p amigo-material-2d-plugin
cargo test -p amigo-material-2d-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/materials/material-2d
rg -n "pub struct|pub enum|pub trait|impl " plugins/materials/material-2d/src
```
