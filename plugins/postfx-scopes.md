# postfx-scopes

Path: `plugins/postfx/scopes`  
Cargo package: `amigo-scopes-plugin`  
Plugin id: `amigo.postfx.scopes`  
Family: `postfx`  
Kind: `tooling`  
Renderable: `False`  
Render participation: `target-consumer`

## Role

postfx plugin `scopes`. Owns a PostFX domain slice or scope/debug contribution, not the WGPU backend unless explicitly connected.

## Manifest capabilities

- provides: `postfx.scopes@1`
- requires: `render.backend@1`

## Manifest slots

- implements: none declared
- requires: `render.backend`
- replaces: none declared

## Manifest targets

- reads: `SceneColor`
- reads: `SceneHighlight`
- reads: `SceneEmissive`
- writes: `DiagnosticsSnapshot`
- contributes: none declared

## Manifest contributions

- emits: none declared
- consumes: none declared

## Manifest diagnostics

- channels: `postfx.scopes`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/postfx/scopes/src/api/mod.rs`
- `plugins/postfx/scopes/src/diagnostics/mod.rs`
- `plugins/postfx/scopes/src/lib.rs`
- `plugins/postfx/scopes/src/participation/mod.rs`
- `plugins/postfx/scopes/src/plugin.rs`
- `plugins/postfx/scopes/src/render_wgpu/mod.rs`
- `plugins/postfx/scopes/src/runtime/mod.rs`
- `plugins/postfx/scopes/src/scene/mod.rs`
- `plugins/postfx/scopes/src/scripting/mod.rs`
- `plugins/postfx/scopes/README.md`
- `plugins/postfx/scopes/plugin.toml`
- `plugins/postfx/scopes/docs/pipeline.md`
- `plugins/postfx/scopes/docs/contributions.md`
- `plugins/postfx/scopes/docs/diagnostics.md`
- `plugins/postfx/scopes/tests/waterfall_tests.rs`

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
cargo check -p amigo-scopes-plugin
cargo test -p amigo-scopes-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/postfx/scopes
rg -n "pub struct|pub enum|pub trait|impl " plugins/postfx/scopes/src
```
