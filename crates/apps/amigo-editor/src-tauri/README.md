# amigo-editor

Tauri backend for the Amigo editor app.

## Responsibility
- Editor window/session backend.
- Mod and asset registry commands.
- Preview, cache, DTO, settings, and sheet services.
- Tauri command surface for the frontend.

## Not here
- Engine runtime ownership.
- Full game-editor logic.
- Web frontend component code.

## Depends on
- amigo-app.
- amigo-modding.
- amigo-scene.
- image.
- serde.
- tauri.
