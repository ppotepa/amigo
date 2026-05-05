# amigo-app-host-winit

Winit host implementation for interactive desktop runs.

## Responsibility
- Own the desktop event loop.
- Forward lifecycle, window, and input events to a host handler.
- Coordinate app runtime with winit and window backend.

## Not here
- Host API definitions.
- Gameplay systems.
- Renderer internals.

## Depends on
- amigo-app-host-api.
- amigo-core.
- amigo-input-api.
- amigo-input-winit.
- amigo-window-api.
- amigo-window-winit.
- raw-window-handle.
- winit.
