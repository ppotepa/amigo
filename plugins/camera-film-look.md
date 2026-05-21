# camera-film-look

Path: `plugins/camera/film-look`  
Cargo package: `amigo-film-look-plugin`  
Plugin id: `amigo.camera.film-look`  
Family: `camera`  
Kind: `target-consumer`  
Renderable: `False`  
Render participation: `target-consumer`

## Role

camera plugin `film-look`. Confirm detailed ownership in plugin.toml, README, and local docs.

## Manifest capabilities

- provides: `camera.film_look.2d@1`
- requires: `camera.frame_context.2d@1`

## Manifest slots

- implements: `camera.film_model.2d`
- requires: `camera.frame_provider.2d`
- replaces: none declared

## Manifest targets

- reads: `SceneColor`
- reads: `CameraArtifactLayer`
- writes: `FinalComposite`
- contributes: none declared

## Manifest contributions

- emits: none declared
- consumes: none declared

## Manifest diagnostics

- channels: `film-look.composite`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/camera/film-look/src/api/mod.rs`
- `plugins/camera/film-look/src/api/profile.rs`
- `plugins/camera/film-look/src/api/response.rs`
- `plugins/camera/film-look/src/diagnostics/format.rs`
- `plugins/camera/film-look/src/diagnostics/mod.rs`
- `plugins/camera/film-look/src/lib.rs`
- `plugins/camera/film-look/src/manifest.rs`
- `plugins/camera/film-look/src/participation/mod.rs`
- `plugins/camera/film-look/src/plugin.rs`
- `plugins/camera/film-look/src/render_wgpu/mod.rs`
- `plugins/camera/film-look/src/render_wgpu/pass.rs`
- `plugins/camera/film-look/src/runtime/film_grain.rs`
- `plugins/camera/film-look/src/runtime/mod.rs`
- `plugins/camera/film-look/src/runtime/resolve.rs`
- `plugins/camera/film-look/src/scene/descriptors.rs`
- `plugins/camera/film-look/src/scene/document.rs`
- `plugins/camera/film-look/README.md`
- `plugins/camera/film-look/plugin.toml`
- `plugins/camera/film-look/docs/pipeline.md`
- `plugins/camera/film-look/docs/contributions.md`
- `plugins/camera/film-look/docs/diagnostics.md`
- `plugins/camera/film-look/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-composite-plugin`
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
cargo check -p amigo-film-look-plugin
cargo test -p amigo-film-look-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/camera/film-look
rg -n "pub struct|pub enum|pub trait|impl " plugins/camera/film-look/src
```
