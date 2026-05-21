# postfx-color-grading

Path: `plugins/postfx/color-grading`  
Cargo package: `amigo-color-grading-plugin`  
Plugin id: `amigo.postfx.color-grading`  
Family: `postfx`  
Kind: `target-consumer`  
Renderable: `False`  
Render participation: `target-consumer`

## Role

postfx plugin `color-grading`. Owns a PostFX domain slice or scope/debug contribution, not the WGPU backend unless explicitly connected.

## Manifest capabilities

- provides: `postfx.color-grading@1`
- requires: `render.backend@1`

## Manifest slots

- implements: none declared
- requires: `render.backend`
- replaces: none declared

## Manifest targets

- reads: `SceneColor`
- writes: `SceneColor`
- contributes: none declared

## Manifest contributions

- emits: none declared
- consumes: none declared

## Manifest diagnostics

- channels: `postfx.color-grading`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/postfx/color-grading/src/api/mod.rs`
- `plugins/postfx/color-grading/src/diagnostics/mod.rs`
- `plugins/postfx/color-grading/src/lib.rs`
- `plugins/postfx/color-grading/src/participation/mod.rs`
- `plugins/postfx/color-grading/src/plugin.rs`
- `plugins/postfx/color-grading/src/render_wgpu/mod.rs`
- `plugins/postfx/color-grading/src/runtime/mod.rs`
- `plugins/postfx/color-grading/src/scene/mod.rs`
- `plugins/postfx/color-grading/src/scripting/mod.rs`
- `plugins/postfx/color-grading/README.md`
- `plugins/postfx/color-grading/plugin.toml`
- `plugins/postfx/color-grading/docs/pipeline.md`
- `plugins/postfx/color-grading/docs/contributions.md`
- `plugins/postfx/color-grading/docs/diagnostics.md`
- `plugins/postfx/color-grading/tests/waterfall_tests.rs`

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
cargo check -p amigo-color-grading-plugin
cargo test -p amigo-color-grading-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/postfx/color-grading
rg -n "pub struct|pub enum|pub trait|impl " plugins/postfx/color-grading/src
```
