# postfx-bloom

Path: `plugins/postfx/bloom`  
Cargo package: `amigo-bloom-plugin`  
Plugin id: `amigo.postfx.bloom`  
Family: `postfx`  
Kind: `target-consumer`  
Renderable: `False`  
Render participation: `target-consumer`

## Role

postfx plugin `bloom`. Owns a PostFX domain slice or scope/debug contribution, not the WGPU backend unless explicitly connected.

## Manifest capabilities

- provides: `postfx.bloom@1`
- requires: none declared

## Manifest slots

- implements: none declared
- requires: `render.backend`
- replaces: none declared

## Manifest targets

- reads: `SceneEmissive`
- reads: `SceneColor`
- writes: `SceneColor`
- contributes: none declared

## Manifest contributions

- emits: none declared
- consumes: none declared

## Manifest diagnostics

- channels: `postfx.bloom`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/postfx/bloom/src/api/mod.rs`
- `plugins/postfx/bloom/src/diagnostics/mod.rs`
- `plugins/postfx/bloom/src/lib.rs`
- `plugins/postfx/bloom/src/participation/mod.rs`
- `plugins/postfx/bloom/src/plugin.rs`
- `plugins/postfx/bloom/src/render_wgpu/mod.rs`
- `plugins/postfx/bloom/src/runtime/mod.rs`
- `plugins/postfx/bloom/src/scene/mod.rs`
- `plugins/postfx/bloom/src/scripting/mod.rs`
- `plugins/postfx/bloom/README.md`
- `plugins/postfx/bloom/plugin.toml`
- `plugins/postfx/bloom/docs/pipeline.md`
- `plugins/postfx/bloom/docs/contributions.md`
- `plugins/postfx/bloom/docs/diagnostics.md`
- `plugins/postfx/bloom/tests/waterfall_tests.rs`

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
cargo check -p amigo-bloom-plugin
cargo test -p amigo-bloom-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/postfx/bloom
rg -n "pub struct|pub enum|pub trait|impl " plugins/postfx/bloom/src
```
