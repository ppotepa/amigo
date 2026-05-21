# lighting-relight-2d

Path: `plugins/lighting/relight-2d`  
Cargo package: `amigo-relight-2d-plugin`  
Plugin id: `amigo.lighting.relight-2d`  
Family: `lighting`  
Kind: `target-consumer`  
Renderable: `False`  
Render participation: `target-consumer`

## Role

lighting plugin `relight-2d`. Confirm detailed ownership in plugin.toml, README, and local docs.

## Manifest capabilities

- provides: `lighting.relight.2d@1`
- requires: `lighting.light.2d@1`

## Manifest slots

- implements: none declared
- requires: `render.backend`
- replaces: none declared

## Manifest targets

- reads: `SceneColor`
- reads: `SceneLighting`
- reads: `LightMap`
- writes: `SceneColor`
- contributes: none declared

## Manifest contributions

- emits: none declared
- consumes: `{"domain": "lighting", "type": "LightContribution2d", "policy": "ExplicitOnly"}`

## Manifest diagnostics

- channels: `relight-2d.targets`
- channels: `relight-2d.diagnostics`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/lighting/relight-2d/src/api/mod.rs`
- `plugins/lighting/relight-2d/src/debug_view.rs`
- `plugins/lighting/relight-2d/src/diagnostics/mod.rs`
- `plugins/lighting/relight-2d/src/lib.rs`
- `plugins/lighting/relight-2d/src/participation/mod.rs`
- `plugins/lighting/relight-2d/src/plugin.rs`
- `plugins/lighting/relight-2d/src/render_wgpu/mod.rs`
- `plugins/lighting/relight-2d/src/runtime/mod.rs`
- `plugins/lighting/relight-2d/src/scene/mod.rs`
- `plugins/lighting/relight-2d/src/scripting/mod.rs`
- `plugins/lighting/relight-2d/README.md`
- `plugins/lighting/relight-2d/plugin.toml`
- `plugins/lighting/relight-2d/docs/pipeline.md`
- `plugins/lighting/relight-2d/docs/contributions.md`
- `plugins/lighting/relight-2d/docs/diagnostics.md`
- `plugins/lighting/relight-2d/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-camera`
- `amigo-plugin-api`
- `amigo-render-api`

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
cargo check -p amigo-relight-2d-plugin
cargo test -p amigo-relight-2d-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/lighting/relight-2d
rg -n "pub struct|pub enum|pub trait|impl " plugins/lighting/relight-2d/src
```
