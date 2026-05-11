# amigo-session

`amigo-session` is the host-independent runtime session layer for Amigo.

It sits above `amigo-runtime` and below concrete hosts such as `amigo-app`,
the future editor, headless validation tools, and scene preview tools.

## Owns
- `RuntimeSession`
- `SceneSessionService`
- `RenderSessionService`
- `SchedulerSessionService`
- `ScriptSessionService`
- session lifecycle summaries and host-independent lifecycle state
- runtime session profiles and bootstrap DTOs

## Does not own
- window or event-loop control
- WGPU surface ownership
- app-specific startup UX
- duplicate runtime systems or `v2` paths

## Bootstrap boundary
`RuntimeSession` is the ownership boundary after host-specific bootstrap.

Concrete hosts may still assemble the low-level `Runtime`, but new host/editor
facing code should prefer `RuntimeSessionBootstrap<TSummary>` instead of raw
`(Runtime, Summary)` tuples.

## P0.1 closure status
- scene load and queue now have session-aware boundaries
- scene command dispatch now has a session-aware boundary
- scene clear now has a session-aware boundary
- render lifecycle now has a session-aware boundary
- scheduler/system phases now have a session-aware boundary
- script dispatch now has a session-aware boundary
- remaining app-owned domain handlers remain migration seams until P0.2

## Session boundaries exposed by `RuntimeSession`
- scene lifecycle: load, hydration queue, command, clear, error state
- render lifecycle: extract, composition, graph build, submit, present, error state
- scheduler lifecycle: per-phase begin, complete, and error state
- script lifecycle: dispatch begin, complete, and error state

## Current migration shape
- `amigo-app` still owns concrete scene/runtime/render/system/script implementations.
- Those implementations now update shared lifecycle state through session services.
- Future passes should move domain-owned handlers and extractors out of `amigo-app` without changing the host/session boundary.

## P0.2 Foundation
P0.2 introduces runtime domain contribution descriptors.
The registry is the boundary for moving app-owned domain logic into engine/domain crates.
Initial providers may wrap legacy app-owned implementations.
`RuntimeSession` exposes contribution summaries (`runtime_contribution_summary`,
`runtime_diagnostics_summary`, `runtime_metadata_summary`) to support host/editor
consumption without app-only assumptions.

## P0.2 cleanup posture
- contribution descriptors classify ownership as `app.legacy`, `app.host`, or domain-owned.
- app-owned `LegacyApp*Provider` adapters are expected only as temporary migration seams.
- after dedicated domain providers are added, legacy adapters should be removed.
- app should remain host/glue: window/input/wgpu wiring and orchestration only.
