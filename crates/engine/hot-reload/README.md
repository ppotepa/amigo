# amigo-hot-reload

Hot-reload coordination for assets and scene documents.

## Responsibility
- Convert file-watch events into reload requests.
- Track reload intent for runtime systems.
- Keep hot-reload orchestration separate from file watching.

## Not here
- Concrete filesystem watcher implementation.
- Asset parsing.
- Scene hydration.

## Depends on
- amigo-core.
- amigo-runtime.
