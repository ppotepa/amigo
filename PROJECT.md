# PROJECT.md

This document is the canonical short project-state overview for Amigo.

Use this file to orient humans and agents before they open deeper architecture docs. Keep operational agent rules in `AGENTS.md`; keep detailed architecture in `docs/architecture/**`.

## Project identity

Amigo is a mod-first Rust monorepo for a 2D/3D runtime engine, launcher, in-game tooling, plugins, and authored mods.

Current design direction:

```text
thin app host
  -> runtime bundle composition
  -> domain plugins
  -> scene/hydration contracts
  -> render-api contracts
  -> render-wgpu backend execution
  -> diagnostics/devtools
```

The project is still in an active refactor phase. Clean final architecture is preferred over preserving compatibility paths.

## Current architecture map

```text
crates/apps/app              runtime host / bootstrap only
crates/apps/launcher         launcher / profile selection
crates/runtime/bundles       runtime composition and backend bridges
crates/engine/runtime        plugin and system runtime contracts
crates/engine/session        session/frame scheduling services
crates/engine/scene          scene documents, hydration, metadata, commands
crates/engine/render-api     render contracts, frame graph, post-fx models
crates/engine/render-wgpu    WGPU backend implementation
crates/engine/camera         shared camera contracts/services
crates/engine/devtools       dev console, debug overlay, diagnostics
crates/engine/editor-api     editor contracts
crates/engine/editor-session editor session contracts
crates/engine/editor-authoring authoring graph/cache services
crates/engine/editor-ingame  runtime in-game editor overlay/mockup
plugins/                     domain-owned plugin implementations
mods/                        authored content, scenes, scripts, assets
```

## Stable architecture rules

- `apps/app` must stay a thin host.
- `runtime/bundles` composes; it does not own domain behavior.
- `render-api` owns renderer-facing contracts, not WGPU implementation.
- `render-wgpu` executes contracts; it should not infer domain intent.
- Plugins own domain semantics through contribution/candidate/target waterfalls.
- Mods own content, not engine behavior.
- Do not add `v2`, legacy, compatibility, or fallback paths unless the task explicitly requires a temporary diagnostic isolation.

## Active refactor focus

The app-centric refactor is mostly complete. The highest remaining risk is central renderer coupling, especially in PostFX and camera optics.

Known high-risk areas:

```text
crates/engine/render-wgpu/src/renderer/service/post_fx/registry.rs
crates/engine/render-wgpu/src/renderer/service/model.rs
crates/engine/render-wgpu/src/renderer/service/init.rs
crates/engine/render-wgpu/src/renderer/service/render/scoped_post_fx.rs
crates/engine/render-wgpu/src/renderer/service/render/visual_debug.rs
crates/engine/render-wgpu/src/renderer/service/texture_batches.rs
crates/engine/render-api/src/post_fx_model/flat_metadata.rs
crates/engine/scene/src/component_metadata.rs
```

The next architecture target is descriptor/registry-driven PostFX execution instead of central `match PostFx2d` dispatch.

## Camera optics target state

Camera optical artifacts should be produced through explicit optical contracts, not renderer guesses.

Current useful concepts:

```text
CameraOpticalResponse2d
CameraOpticalCoverage2d
CameraOpticalCandidate2d
CameraOpticalRenderTargetPlan
SceneHighlight
SceneEmissive
```

Lighting, lightmaps, materials, particles, text, vector coverage, and layered images should declare contributions/candidates explicitly when they participate in camera artifacts.

## Documentation map

Canonical docs:

```text
README.md                                      human entrypoint
docs/architecture/runtime-refactor-status.md  runtime refactor status
docs/architecture/runtime-bundles.md          runtime bundle rules
docs/architecture/render-composition.md       render composition rules
docs/architecture/camera-driven-2d-pipeline.md camera-driven 2D rules
docs/architecture/plugins/README.md           plugin architecture overview
docs/architecture/plugins/canonical-tree.md   canonical plugin tree
docs/architecture/plugins/waterfall.md        plugin waterfall model
```

Operational docs:

```text
AGENTS.md             agent workflow and repository rules
codemap.index.md      codemap taxonomy/navigation guide
```

Non-canonical / cleanup candidates:

```text
arch.md               appears to be a pasted historical refactor plan; do not treat as canonical until cleaned or archived
```

If `PROJECT.md` was missing before this file was added, use this document as the canonical replacement for project-state notes.

## Plugin documentation quality rule

Many plugin docs may start as placeholders. When a plugin is touched, bring its local docs up to useful quality instead of leaving only a heading.

Touched plugin documentation should explain:

```text
what the plugin owns
what it contributes
what it consumes
which diagnostics it emits
which tests validate its waterfall
```

Do not mass-update all placeholder docs in one unrelated task.

## Recommended validation commands

Docs-only change:

```powershell
git diff --check
```

Targeted code check:

```powershell
cargo check -p <crate>
```

Targeted tests:

```powershell
cargo test -p <crate> <filter>
```

Codemap navigation:

```powershell
cargo build -p amigo-codemap
Copy-Item target\debug\amigo-codemap.exe target\debug\amigo-codemap-stable.exe
$cm = "target\debug\amigo-codemap-stable.exe"
& $cm brief
& $cm change-plan "<task>" --limit 20
& $cm open-set "<topic>" --why --limit 20
```
