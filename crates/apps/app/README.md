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
