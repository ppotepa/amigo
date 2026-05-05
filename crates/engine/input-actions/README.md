# amigo-input-actions

Named input action mapping over raw input state.

## Responsibility
- Resolve actions and axes from platform-neutral input.
- Store active input maps.
- Provide gameplay-facing input queries.

## Not here
- Winit event conversion.
- Window management.
- Gameplay behavior code.

## Depends on
- amigo-core.
- amigo-input-api.
- amigo-runtime.
