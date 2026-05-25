# PROJECT.md

This is the canonical short project-state overview for Amigo.

Use this file to orient humans and agents before opening deeper architecture
docs. Keep operational agent rules in `AGENTS.md`; keep detailed architecture
notes in `docs/architecture/**`.

## Project identity

Amigo is a mod-first Rust monorepo for a 2D/3D runtime engine, launcher,
in-game tooling, plugins, and authored mods.

Current direction:

```text
thin app host
  -> runtime bundle composition
  -> domain plugins
  -> scene/hydration contracts
  -> render-api contracts
  -> render-wgpu backend execution
  -> diagnostics/devtools
```

The project is still in active refactor. Prefer clean final architecture over
compatibility paths, duplicate `v2` systems, or fallback behavior that hides
missing contracts.

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

- `apps/app` stays a thin host.
- `runtime/bundles` composes; it does not own domain behavior.
- `render-api` owns renderer-facing contracts, not WGPU implementation.
- `render-wgpu` executes contracts; it should not infer domain intent.
- Plugins own domain semantics through contribution/candidate/target waterfalls.
- Mods own content, not engine behavior.
- Do not add `legacy`, `v2`, compatibility, or fallback paths unless a task
  explicitly requires temporary diagnostic isolation.

## Active risk

PostFX and camera optics remain the highest-risk architecture area. New work
should move toward descriptor/registry-driven execution and explicit optical
contracts instead of renderer-side effect switches or object-existence guesses.

## Canonical documentation map

```text
README.md                         human entrypoint
PROJECT.md                        short project-state overview
AGENTS.md                         agent workflow and repository rules
docs/architecture/**              architecture source of truth
plugins/<family>/<plugin>/docs/   plugin-owned domain documentation
```

Generated crate/plugin inventory snapshots and pasted refactor plans are not
canonical documentation. Use codemap and Cargo metadata for current inventory.
