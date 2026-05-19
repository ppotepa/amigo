# Runtime Refactor Status

## Done

- `apps/app/src/scene_runtime/handlers` removed.
- `apps/app/src/script_runtime/handlers` removed.
- Domain systems moved out of app.
- App no longer depends directly on 2D/3D/audio/UI domain crates.
- Devtools moved to `crates/engine/devtools`.
- Runtime bundles moved to `crates/runtime/bundles`.
- Camera API exists in `crates/engine/camera`.
- Editor placeholder APIs exist in `crates/engine/editor-api` and `crates/engine/editor-session`.

## Current Seams

- `apps/app/src/render_runtime.rs` still orchestrates host render frame submission.
- `crates/runtime/bundles/src/wgpu_render_extractors` is a backend bridge.
- `CompositionLayer` and `RenderSpace` are integrated as transition metadata while old pass planning remains.
- Editor APIs are placeholders only. No full editor application exists.

## Rules

- Do not add domain imports to `apps/app`.
- Do not move domain logic into `runtime/bundles`.
- Do not add UI framework dependencies to `editor-api` or `editor-session`.
- Do not build `apps/editor` yet.
