# devtools-codemap

Path: `plugins/devtools/codemap`  
Cargo package: `amigo-codemap-plugin`  
Plugin id: `amigo.devtools.codemap`  
Family: `devtools`  
Kind: `tooling`  
Renderable: `False`  
Render participation: `none`

## Role

devtools plugin `codemap`. Confirm detailed ownership in plugin.toml, README, and local docs.

## Manifest capabilities

- provides: `devtools.codemap.index@1`
- provides: `devtools.diagnostics.provider@1`
- requires: none declared

## Manifest slots

- implements: `codemap.index_provider`
- implements: `diagnostics.provider`
- requires: none declared
- replaces: none declared

## Manifest targets

- reads: `DiagnosticsSnapshot`
- writes: `DiagnosticsSnapshot`
- contributes: none declared

## Manifest contributions

- emits: none declared
- consumes: none declared

## Manifest diagnostics

- channels: `devtools.codemap.index`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/devtools/codemap/src/api/mod.rs`
- `plugins/devtools/codemap/src/diagnostics/mod.rs`
- `plugins/devtools/codemap/src/lib.rs`
- `plugins/devtools/codemap/src/participation/mod.rs`
- `plugins/devtools/codemap/src/plugin.rs`
- `plugins/devtools/codemap/src/render_wgpu/mod.rs`
- `plugins/devtools/codemap/src/runtime/mod.rs`
- `plugins/devtools/codemap/src/scene/mod.rs`
- `plugins/devtools/codemap/src/scripting/mod.rs`
- `plugins/devtools/codemap/README.md`
- `plugins/devtools/codemap/plugin.toml`
- `plugins/devtools/codemap/docs/pipeline.md`
- `plugins/devtools/codemap/docs/contributions.md`
- `plugins/devtools/codemap/docs/diagnostics.md`
- `plugins/devtools/codemap/tests/waterfall_tests.rs`

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
cargo check -p amigo-codemap-plugin
cargo test -p amigo-codemap-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/devtools/codemap
rg -n "pub struct|pub enum|pub trait|impl " plugins/devtools/codemap/src
```
