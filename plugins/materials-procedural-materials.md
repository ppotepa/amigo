# materials-procedural-materials

Path: `plugins/materials/procedural-materials`  
Cargo package: `amigo-procedural-materials-plugin`  
Plugin id: `amigo.materials.procedural-materials`  
Family: `materials`  
Kind: `target-consumer`  
Renderable: `False`  
Render participation: `target-writer`

## Role

materials plugin `procedural-materials`. Confirm detailed ownership in plugin.toml, README, and local docs.

## Manifest capabilities

- provides: `materials.procedural-materials@1`
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

- channels: `procedural-materials.targets`
- channels: `procedural-materials.diagnostics`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/materials/procedural-materials/src/api/mod.rs`
- `plugins/materials/procedural-materials/src/diagnostics/mod.rs`
- `plugins/materials/procedural-materials/src/lib.rs`
- `plugins/materials/procedural-materials/src/participation/mod.rs`
- `plugins/materials/procedural-materials/src/plugin.rs`
- `plugins/materials/procedural-materials/src/render_wgpu/mod.rs`
- `plugins/materials/procedural-materials/src/runtime/mod.rs`
- `plugins/materials/procedural-materials/src/scene/mod.rs`
- `plugins/materials/procedural-materials/src/scripting/mod.rs`
- `plugins/materials/procedural-materials/README.md`
- `plugins/materials/procedural-materials/plugin.toml`
- `plugins/materials/procedural-materials/docs/pipeline.md`
- `plugins/materials/procedural-materials/docs/contributions.md`
- `plugins/materials/procedural-materials/docs/diagnostics.md`
- `plugins/materials/procedural-materials/tests/waterfall_tests.rs`

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
cargo check -p amigo-procedural-materials-plugin
cargo test -p amigo-procedural-materials-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/materials/procedural-materials
rg -n "pub struct|pub enum|pub trait|impl " plugins/materials/procedural-materials/src
```
