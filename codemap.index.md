<!-- @codemap anchor:codemap-index domain:codemap role:taxonomy-doc priority:P0 layer:docs tags:taxonomy,navigation,workflow -->

# Codemap Index

`codemap.index.md` is the human-readable guide for Amigo codemap anchors and taxonomy. It is source documentation, not a generated snapshot.

Generated codemap data lives in:

- `.amigo/codemap.anchors.generated.json`
- `.amigo/codemap.coverage.generated.md` when regenerated
- `.amigo/codemap.snapshot.json`

The machine-readable taxonomy lives in:

- `.amigo/codemap.taxonomy.yml`

## Anchor Format

Use anchors for stable navigation points that agents should find early:

```text
@codemap anchor:<name> domain:<domain> role:<role> priority:<priority> layer:<layer> tags:<tag1>,<tag2>
```

Example:

```rust
// @codemap anchor:codemap-command-dispatch domain:codemap role:dispatcher priority:P0 layer:tool tags:cli,dispatch
```

## Priorities

| Priority | Meaning | Use for |
| --- | --- | --- |
| P0 | Critical navigation point | Entrypoints, dispatchers, registries, root contracts |
| P1 | Important domain file | Models, adapters, reports, command modules |
| P2 | Indexed file-level anchor | Broad generated or low-touch coverage |
| P3 | Low-level helper | Repeated navigation value only |
| PX | Experimental | Temporary or unstable feature area |

## Layers

| Layer | Typical paths |
| --- | --- |
| app | `crates/apps/app/**` |
| runtime | `crates/runtime/**` |
| engine | `crates/engine/**` |
| plugin | `plugins/**` |
| platform | `crates/platform/**` |
| scripting | `crates/scripting/**` |
| tool | `crates/tools/**` |
| mod | `mods/**` |
| docs | `*.md`, `docs/**`, `reference/**`, `workflows/**` |

## Domains

Keep domains aligned with current ownership boundaries:

| Domain | Main paths |
| --- | --- |
| codemap | `crates/tools/amigo-codemap/**`, codemap metadata |
| runtime | `crates/runtime/bundles/**`, runtime composition bridges |
| engine-scene | `crates/engine/scene/**` |
| render-api | `crates/engine/render-api/**` |
| render-wgpu | `crates/engine/render-wgpu/**` |
| camera | `plugins/camera/**`, `crates/engine/camera/**` |
| gfx | `plugins/gfx/**` |
| lighting | `plugins/lighting/**` |
| postfx | `plugins/postfx/**` |
| vfx | `plugins/vfx/**` |
| docs | Canonical repository documentation |
| playground | `mods/playground-*` |

Add or refine domains in `.amigo/codemap.taxonomy.yml` when a new ownership area becomes permanent.

## Roles

Common roles:

- `entrypoint`, `bootstrap`, `dispatcher`, `registry`
- `model`, `contract`, `types`, `adapter`
- `scanner-entrypoint`, `file-indexer`, `symbol-indexer`, `anchor-parser`
- `report`, `workflow-report`, `impact-report`, `verify-report`
- `file-ops`, `patch-apply`, `ops-plan`, `slice-report`
- `scene-yaml`, `scene-script`, `mod-manifest`, `docs`

## Workflow

Use codemap before opening large files:

```powershell
cargo build -p amigo-codemap
Copy-Item target\debug\amigo-codemap.exe target\debug\amigo-codemap-stable.exe
$cm = "target\debug\amigo-codemap-stable.exe"

& $cm brief
& $cm changes --compact --hide-generated --limit 20
& $cm change-plan "<task>" --limit 20
& $cm open-set "<symbols / paths / topic>" --why --limit 20
```

Use symbol or range reads after `open-set`:

```powershell
& $cm symbols --file <path> --metadata --limit 40
& $cm slice <path> --symbol <symbol>
```

## Maintenance

When adding or moving important engine, runtime, plugin, app, tool, or mod surfaces:

1. Add manual P0/P1 anchors for meaningful entrypoints, registries, contracts, adapters, command modules, scene YAML files, or scripts.
2. Update `.amigo/codemap.taxonomy.yml` only when the feature introduces a durable domain, role, layer, or scoring rule.
3. Regenerate generated codemap files when anchor coverage changes:

```powershell
& $cm anchors --write
& $cm anchor-check
```

Manual P0/P1 anchors explain intent. Generated P2 anchors provide coverage and should not replace meaningful manual anchors.
