# lighting-lightmaps-2d

Path: `plugins/lighting/lightmaps-2d`  
Cargo package: `amigo-lightmaps-2d-plugin`  
Plugin id: `amigo.lighting.lightmaps-2d`  
Family: `lighting`  
Kind: `semantic-source`  
Renderable: `False`  
Render participation: `none`

## Role

lighting plugin `lightmaps-2d`. Owns lightmap channel semantics and target/source contribution data.

## Manifest capabilities

- provides: `lighting.lightmaps.2d@1`
- requires: none declared

## Manifest slots

- implements: none declared
- requires: none declared
- replaces: none declared

## Manifest targets

- reads: none declared
- writes: `LightMap`
- contributes: `SceneLighting`
- contributes: `SceneHighlight`
- contributes: `SceneEmissive`

## Manifest contributions

- emits: `{"domain": "camera.optics", "type": "CameraOpticsContribution2d", "policy": "ExplicitOnly"}`
- consumes: none declared

## Manifest diagnostics

- channels: `lightmaps-2d.contributions`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/lighting/lightmaps-2d/src/api/mod.rs`
- `plugins/lighting/lightmaps-2d/src/diagnostics/mod.rs`
- `plugins/lighting/lightmaps-2d/src/lib.rs`
- `plugins/lighting/lightmaps-2d/src/participation/mod.rs`
- `plugins/lighting/lightmaps-2d/src/plugin.rs`
- `plugins/lighting/lightmaps-2d/src/render_wgpu/mod.rs`
- `plugins/lighting/lightmaps-2d/src/runtime/mod.rs`
- `plugins/lighting/lightmaps-2d/src/scene/mod.rs`
- `plugins/lighting/lightmaps-2d/src/scripting/mod.rs`
- `plugins/lighting/lightmaps-2d/README.md`
- `plugins/lighting/lightmaps-2d/plugin.toml`
- `plugins/lighting/lightmaps-2d/docs/pipeline.md`
- `plugins/lighting/lightmaps-2d/docs/contributions.md`
- `plugins/lighting/lightmaps-2d/docs/diagnostics.md`
- `plugins/lighting/lightmaps-2d/tests/waterfall_tests.rs`

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
cargo check -p amigo-lightmaps-2d-plugin
cargo test -p amigo-lightmaps-2d-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/lighting/lightmaps-2d
rg -n "pub struct|pub enum|pub trait|impl " plugins/lighting/lightmaps-2d/src
```
