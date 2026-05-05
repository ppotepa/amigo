# amigo-scripting-rhai

Rhai scripting backend for gameplay and tooling scripts.

## Responsibility
- Bind engine services into Rhai.
- Load and register script packages.
- Drive script lifecycle callbacks.
- Expose script-facing handles and world APIs.

## Not here
- Generic scripting contracts.
- Domain service ownership.
- Scene document format.

## Depends on
- amigo-scripting-api.
- amigo-runtime.
- amigo-scene.
- 2D domain crates.
- input, state, modding, asset, UI, and event crates.
- rhai.
