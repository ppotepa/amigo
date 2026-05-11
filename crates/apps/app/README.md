# amigo-app

Primary Amigo application runtime.

## Responsibility
- Bootstrap engine services.
- Load mods, scenes, scripts, assets, and runtime plugins.
- Drive update, rendering, scripting, audio, input, and scene hydration.
- Coordinate hosted/headless app execution.

## Not here
- Platform event loop implementation.
- Concrete renderer backend internals.
- Editor UI.

## Depends on
- Most engine, platform, rendering, scripting, audio, UI, and domain crates.

## Session boundary
- `amigo-app` is the concrete host glue.
- Reusable runtime session orchestration belongs in `amigo-session`.
- New host-independent startup, per-frame session state, or render-request
  plumbing should move toward `crates/engine/session` instead of growing here.

## Bootstrap migration status
- `amigo-app` still contains the concrete bootstrap implementation.
- Hosted execution now flows through `RuntimeSession`.
- Prefer `bootstrap_session_default` and `bootstrap_session_with_options` for
  new code.

## Scene session migration status

Hosted bootstrap now copies loaded scene metadata into `RuntimeSession::scene_session`.

This is still a boundary step only. App-owned scene paths remain active migration seams:

- `scene_runtime::load_scene_document_for_mod`
- `scene_runtime::queue_scene_document_hydration`
- `scene_runtime::apply_scene_command`
- `scene_runtime::handlers::*`

## Scene lifecycle migration status

`RuntimeSession::scene_session` now records lifecycle state after app bootstrap:

- loaded scene metadata is copied into `SceneSession`,
- hydration queueing is marked on the session,
- processed scene commands are counted on the session.

The app still owns the real scene runtime implementation until the next scene lifecycle migration passes.

## Runtime service lifecycle sync

`amigo-app` also registers `SceneSessionService` in bootstrap runtime services so
app-owned scene command handlers can update session state directly through the shared
runtime service.

Covered lifecycle updates:

- loaded scene metadata
- hydration queueing
- scene command success/failure
- scene clear
- scene lifecycle errors

## Scene load/queue migration status

Bootstrap no longer calls scene lifecycle recorders directly. Startup scene
loading and hydration queueing now go through session-aware adapters:

- `load_scene_document_for_session`
- `queue_scene_document_hydration_for_session`

The old app-owned loader and queue implementation remain as internal migration
seams until the real scene runtime is moved into `amigo-session` or
domain-owned scene contributions.
