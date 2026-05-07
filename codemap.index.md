<!-- @codemap anchor:codemap-index domain:codemap role:taxonomy-doc priority:P0 layer:docs tags:taxonomy,navigation,workflow -->

# Codemap Index

This document is the human-readable navigation guide for `@codemap` anchors in the Amigo repository.

It is not a generated full file list. Generated data lives in:

- `.amigo/codemap.anchors.generated.json`
- `.amigo/codemap.coverage.generated.md`

The machine-readable taxonomy lives in:

- `.amigo/codemap.taxonomy.yml`

## Purpose

`@codemap` anchors are stable navigation points for humans and LLM agents. They describe what a file or region is, which domain it belongs to, how important it is, and whether it should be opened early during a task.

Default format:

```text
@codemap anchor:<name> domain:<domain> role:<role> priority:<priority> layer:<layer> tags:<tag1>,<tag2>
```

Examples:

```rust
// @codemap anchor:codemap-command-dispatch domain:codemap role:dispatcher priority:P0 layer:tool tags:cli,dispatch
```

```tsx
// @codemap anchor:ui-document-editor-root domain:ui-document role:editor-root priority:P0 layer:app tags:ui,editor
```

```yaml
# @codemap anchor:tar-main-menu-scene domain:they-are-rotten role:scene-yaml priority:P1 layer:mod tags:scene,menu
```

## Priorities

| Priority | Meaning | Use for |
|---|---|---|
| P0 | Critical navigation point | entrypoint, dispatcher, registry, root model, command map |
| P1 | Important domain file | editor root, backend command module, scanner, report, model |
| P2 | Indexed file-level anchor | broad repo coverage |
| P3 | Low-level helper | repeated navigation value only |
| PX | Experimental | unstable or temporary feature |

## Layers

| Layer | Meaning | Typical paths |
|---|---|---|
| tool | Developer tools | `crates/tools/**` |
| app | Editor/frontend application | `crates/apps/amigo-editor/src/**` |
| app-backend | Tauri backend | `crates/apps/amigo-editor/src-tauri/**` |
| runtime-app | Runtime app shell | `crates/apps/app/**` |
| engine | Engine crates | `crates/engine/**`, `crates/ui/**` |
| platform | Platform abstraction | `crates/platform/**` |
| scripting | Rhai/API scripting | `crates/scripting/**` |
| mod | Mods/scenes/assets | `mods/**` |
| docs | Documentation | `*.md`, `docs/**` |

## Domains

| Domain | Meaning | Main paths |
|---|---|---|
| codemap | Codemap tool itself | `crates/tools/amigo-codemap/**` |
| workspace | Amigo editor workspace shell | `crates/apps/amigo-editor/src/main-window/**`, `src/dock/**` |
| editor-components | Editor component/surface definitions | `crates/apps/amigo-editor/src/editor-components/**` |
| ui-document | UI Document Editor | `crates/apps/amigo-editor/src/editors/ui-document/**` |
| scene-editor | Scene Editor | `crates/apps/amigo-editor/src/features/scenes/editor/**` |
| editor-mode | Backend editor mode | `crates/apps/amigo-editor/src-tauri/src/editor_mode/**` |
| editor-backend | Tauri backend commands/events/windows | `crates/apps/amigo-editor/src-tauri/src/**` |
| editor-api | Frontend Tauri API wrappers and DTOs | `crates/apps/amigo-editor/src/api/**` |
| properties | Inspector/properties system | `crates/apps/amigo-editor/src/properties/**` |
| project | Project tree/items/actions | `crates/apps/amigo-editor/src/features/project/**` |
| assets | Asset browser/registry/import | `crates/apps/amigo-editor/src/features/assets/**`, `src/assets/**` |
| context-dock | Generic context dock UI | `crates/apps/amigo-editor/src/ui/context-dock/**` |
| engine-scene | Scene document/runtime/hydration | `crates/engine/scene/**` |
| ui-core | Runtime UI document/node/layout/state model | `crates/ui/core/**` |
| runtime-app | Runtime app orchestration | `crates/apps/app/**` |
| scripting | Rhai scripting runtime/bindings | `crates/scripting/**` |
| docs | Documentation and workflows | `*.md`, `docs/**` |
| playground | Playground mods | `mods/playground-*` |
| they-are-rotten | They Are Rotten game/mod | `mods/they-are-rotten/**` |

## Roles

Common roles:

- `entrypoint`, `bootstrap`, `dispatcher`, `registry`
- `model`, `types`, `dto-contract`
- `scanner-entrypoint`, `file-indexer`, `symbol-indexer`, `text-indexer`, `anchor-parser`
- `report`, `workflow-report`, `impact-report`, `verify-report`
- `file-ops`, `patch-apply`, `ops-plan`, `symbol-aware-ops`, `slice-report`
- `workspace-surface`, `dock-profile`, `detached-workspace`, `tab-strip`, `right-dock-split`, `window-bridge`
- `editor-root`, `inspector`, `tree`, `palette`, `templates`, `preview`, `renderer`, `style`
- `scene-yaml`, `scene-script`, `mod-manifest`, `docs`

## Default Workflow

Use codemap before opening full files:

```powershell
cargo build -p amigo-codemap

$cm = "target\debug\amigo-codemap.exe"

& $cm changed --group package --limit 20
& $cm change-plan <query> --limit 20
& $cm trace <thing> --limit 20
& $cm open-set <thing> --why --limit 10
& $cm signature <symbol>
& $cm slice <file> --symbol <symbol>
& $cm impact <thing> --limit 30
& $cm verify-plan --changed
```

Anchor-specific workflow:

```powershell
target\debug\amigo-codemap.exe taxonomy
target\debug\amigo-codemap.exe anchors priority:P0 --limit 20
target\debug\amigo-codemap.exe anchors domain:ui-document --limit 20
target\debug\amigo-codemap.exe anchors --write
target\debug\amigo-codemap.exe anchor-check
```

## Navigation Rules

1. Use `trace` for symbols, strings, IDs, commands, scene IDs, asset IDs, CSS classes, and anchors.
2. Use `open-set --why` before opening files.
3. Use `signature` before reading implementation.
4. Use `slice --symbol` instead of reading large files.
5. Use `impact` before editing shared/public code.
6. Use `anchors --write` and `anchor-check` after adding or changing anchors.
7. Compiler and tests remain final truth.

## Feature Maintenance Rule

Codemap is a living navigation index. Feature work should update codemap when it creates new navigation surface.

When adding or moving important engine, editor, runtime, backend, or mod features, update the navigation metadata in the same change:

- Add manual P0/P1 anchors for new entrypoints, dispatchers, registries, root models, DTO contracts, editor roots, backend command modules, scene YAML files, and scene scripts.
- Add or adjust domains, roles, layers, and scoring in `.amigo/codemap.taxonomy.yml` when the feature introduces a new area or concept.
- Regenerate `.amigo/codemap.anchors.generated.json` and `.amigo/codemap.coverage.generated.md`.
- Run `anchor-check` and resolve errors before committing.

Do not treat generated file-level anchors as a replacement for meaningful P0/P1 anchors. Generated anchors provide broad coverage; manual anchors explain intent.

## Maintenance

Run after changing anchors:

```powershell
target\debug\amigo-codemap.exe anchors --write
target\debug\amigo-codemap.exe anchor-check
```
