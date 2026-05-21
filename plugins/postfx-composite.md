# postfx-composite

Path: `plugins/postfx/composite`  
Cargo package: `amigo-composite-plugin`  
Plugin id: `amigo.postfx.composite`  
Family: `postfx`  
Kind: `target-consumer`  
Renderable: `False`  
Render participation: `target-consumer`

## Role

postfx plugin `composite`. Owns PostFX composite integration, diagnostics, and a still-duplicated flat-metadata parsing path; the core `PostFx2d` model now comes from `amigo-render-api`.

## Manifest capabilities

- provides: `postfx.composite@1`
- requires: `render.backend@1`

## Manifest slots

- implements: none declared
- requires: `render.backend`
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

- channels: `postfx.composite`

## Declared docs/tests

Docs: `{"pipeline": "docs/pipeline.md", "contributions": "docs/contributions.md", "diagnostics": "docs/diagnostics.md"}`  
Tests: `{"waterfall": "tests/waterfall_tests.rs", "diagnostics": "tests/diagnostics_tests.rs"}`

## Important files found in snapshot

- `plugins/postfx/composite/src/api/mod.rs`
- `plugins/postfx/composite/src/dev_console.rs`
- `plugins/postfx/composite/src/devtools_console.rs`
- `plugins/postfx/composite/src/diagnostics/mod.rs`
- `plugins/postfx/composite/src/editor_provider.rs`
- `plugins/postfx/composite/src/lib.rs`
- `plugins/postfx/composite/src/model/flat_metadata.rs`
- `plugins/postfx/composite/src/model/mod.rs`
- `plugins/postfx/composite/README.md`
- `plugins/postfx/composite/plugin.toml`
- `plugins/postfx/composite/docs/pipeline.md`
- `plugins/postfx/composite/docs/contributions.md`
- `plugins/postfx/composite/docs/diagnostics.md`
- `plugins/postfx/composite/tests/waterfall_tests.rs`

## Dependencies seen in Cargo.toml

- `amigo-capabilities`
- `amigo-core`
- `amigo-devtools`
- `amigo-editor-api`
- `amigo-editor-ingame`
- `amigo-plugin-api`
- `amigo-render-api`
- `amigo-runtime`
- `amigo-scene`
- `amigo-session`
- `rhai`
- `serde`
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
cargo check -p amigo-composite-plugin
cargo test -p amigo-composite-plugin
```

## Navigation queries

```powershell
rg -n "contribution|candidate|diagnostic|target|capabilit|slot" plugins/postfx/composite
rg -n "pub struct|pub enum|pub trait|impl " plugins/postfx/composite/src
```
