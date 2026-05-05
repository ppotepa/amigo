# amigo-input-winit

Winit-backed input backend.

## Responsibility
- Convert winit keyboard and mouse events.
- Map platform events into amigo-input-api.
- Register input backend services.

## Not here
- Input actions.
- Game controls.
- Window lifecycle ownership.

## Depends on
- amigo-core.
- amigo-runtime.
- amigo-input-api.
- winit.
