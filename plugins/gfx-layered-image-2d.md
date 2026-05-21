# gfx-layered-image-2d

Path: `plugins/gfx/layered-image-2d`  
Cargo package: `amigo-layered-image-2d-plugin`  
Plugin id: `amigo.gfx.layered-image-2d`  
Family: `gfx`  
Kind: `renderable-source`  
Renderable: `True`  
Render participation: `source-renderer`

## Role

gfx plugin `layered-image-2d`. Owns graphics component domain data and render contribution/candidate emission for its component type.

## Manifest capabilities

- provides: `gfx.layered-image.2d@1`
- requires: none declared

## Manifest slots

- implements: none declared
- requires: none declared
- replaces: none declared

## Manifest targets

- reads: none declared
- writes: `SceneColor`
- writes: `SceneAlpha`
- contributes: `SceneHighlight`
- contributes: `SceneEmissive`
- contributes: `SceneDepth`

## Manifest contributions

- emits: `{"domain": "camera.optics", "type": "CameraOpticsContribution2d", "policy": "ExplicitOnly"}`
- emits: `{"domain": "camera.focus_depth", "type": "FocusDepthContribution2d", "policy": "DerivedAtHydration"}`
- consumes: none declared

## Manifest diagnostics

- channels: `layered-image-2d.render`
- channels: `layered-image-2d.contributions`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/gfx/layered-image-2d/src/api/mod.rs`
- `plugins/gfx/layered-image-2d/src/diagnostics/mod.rs`
- `plugins/gfx/layered-image-2d/src/layered_image/asset.rs`
- `plugins/gfx/layered-image-2d/src/layered_image/control.rs`
- `plugins/gfx/layered-image-2d/src/layered_image/dev_console.rs`
- `plugins/gfx/layered-image-2d/src/layered_image/editor_capability.rs`
- `plugins/gfx/layered-image-2d/src/layered_image/editor_provider.rs`
- `plugins/gfx/layered-image-2d/src/layered_image/mod.rs`
- `plugins/gfx/layered-image-2d/src/layered_image/model.rs`
- `plugins/gfx/layered-image-2d/src/layered_image/plugin.rs`
- `plugins/gfx/layered-image-2d/src/layered_image/render_extraction.rs`
- `plugins/gfx/layered-image-2d/src/layered_image/reset.rs`
- `plugins/gfx/layered-image-2d/src/layered_image/runtime_capabilities.rs`
- `plugins/gfx/layered-image-2d/src/layered_image/scene_bridge.rs`
- `plugins/gfx/layered-image-2d/src/layered_image/scene_command.rs`
- `plugins/gfx/layered-image-2d/src/layered_image/script_command.rs`
- `plugins/gfx/layered-image-2d/README.md`
- `plugins/gfx/layered-image-2d/plugin.toml`
- `plugins/gfx/layered-image-2d/docs/pipeline.md`
- `plugins/gfx/layered-image-2d/docs/contributions.md`
- `plugins/gfx/layered-image-2d/docs/diagnostics.md`
- `plugins/gfx/layered-image-2d/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-assets`
- `amigo-capabilities`
- `amigo-composite-plugin`
- `amigo-core`
- `amigo-devtools`
- `amigo-editor-api`
- `amigo-editor-ingame`
- `amigo-math`
- `amigo-plugin-api`
- `amigo-runtime`
- `amigo-runtime-control`
- `amigo-scene`
- `amigo-scripting-api`
- `amigo-session`
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
cargo check -p amigo-layered-image-2d-plugin
cargo test -p amigo-layered-image-2d-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/gfx/layered-image-2d
rg -n "pub struct|pub enum|pub trait|impl " plugins/gfx/layered-image-2d/src
```
