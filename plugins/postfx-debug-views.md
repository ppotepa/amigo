# postfx-debug-views

Path: `plugins/postfx/debug-views`  
Cargo package: `amigo-debug-views-plugin`  
Plugin id: `amigo.postfx.debug-views`  
Family: `postfx`  
Kind: `tooling`  
Renderable: `False`  
Render participation: `target-consumer`

## Role

postfx plugin `debug-views`. Owns a PostFX domain slice or scope/debug contribution, not the WGPU backend unless explicitly connected.

## Manifest capabilities

- provides: `postfx.debug-views@1`
- requires: `render.backend@1`

## Manifest slots

- implements: none declared
- requires: `render.backend`
- replaces: none declared

## Manifest targets

- reads: `SceneColor`
- reads: `SceneDepth`
- reads: `SceneHighlight`
- reads: `SceneEmissive`
- reads: `CameraArtifactLayer`
- writes: `DiagnosticsSnapshot`
- contributes: none declared

## Manifest contributions

- emits: none declared
- consumes: none declared

## Manifest diagnostics

- channels: `postfx.debug-views`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/postfx/debug-views/src/api/mod.rs`
- `plugins/postfx/debug-views/src/diagnostics/mod.rs`
- `plugins/postfx/debug-views/src/lib.rs`
- `plugins/postfx/debug-views/src/participation/mod.rs`
- `plugins/postfx/debug-views/src/plugin.rs`
- `plugins/postfx/debug-views/src/render_wgpu/mod.rs`
- `plugins/postfx/debug-views/src/runtime/mod.rs`
- `plugins/postfx/debug-views/src/scene/mod.rs`
- `plugins/postfx/debug-views/src/scripting/mod.rs`
- `plugins/postfx/debug-views/README.md`
- `plugins/postfx/debug-views/plugin.toml`
- `plugins/postfx/debug-views/docs/pipeline.md`
- `plugins/postfx/debug-views/docs/contributions.md`
- `plugins/postfx/debug-views/docs/diagnostics.md`
- `plugins/postfx/debug-views/tests/waterfall_tests.rs`

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
cargo check -p amigo-debug-views-plugin
cargo test -p amigo-debug-views-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/postfx/debug-views
rg -n "pub struct|pub enum|pub trait|impl " plugins/postfx/debug-views/src
```
