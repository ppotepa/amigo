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
