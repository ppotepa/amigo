# amigo-window-winit

Winit-backed window service implementation.

## Responsibility
- Adapt winit windows into amigo-window-api types.
- Convert winit window events.
- Provide desktop window backend support.

## Not here
- App host lifecycle.
- Rendering backend.
- Input action mapping.

## Depends on
- amigo-core.
- amigo-runtime.
- amigo-window-api.
- winit.
