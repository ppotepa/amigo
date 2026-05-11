# amigo-app

Primary Amigo application runtime.

## Owns
- window and event loop coordination
- WGPU surface ownership and hosted renderer wiring
- host input bridge
- startup UX and hosted/headless bootstrap flow
- dev console UI shell and debug overlay presentation

## Must not own
- new reusable runtime lifecycle logic
- host-independent session orchestration
- engine-level render, scene, scheduler, or script contracts
- editor-facing reusable APIs

New host-independent runtime lifecycle code belongs in `amigo-session` or the
relevant engine/domain crate, not in `amigo-app`.

## Public API boundary
- Prefer `bootstrap_session_default` and `bootstrap_session_with_options` for new bootstrap code.
- Prefer `run_hosted_once` and `run_hosted_with_options` for hosted execution.
- The raw bootstrap implementation remains crate-internal as a migration seam.

## P0.1 closure status
- `RuntimeSession` is the central lifecycle boundary used by hosted app flow.
- Startup scene load and queue go through `load_scene_document_for_session` and `queue_scene_document_hydration_for_session`.
- Startup orchestration routes scene commands through `apply_scene_command_for_session`.
- Render lifecycle goes through `build_render_frame_for_session` and updates `RenderSessionService` through `RuntimeSession`.
- System phases go through `run_app_system_phase_for_session` and update `SchedulerSessionService`.
- Script dispatch goes through `dispatch_script_command_for_session` and updates `ScriptSessionService`.
- Remaining app-owned scene/script/orchestration helpers are explicit migration seams.

## Migration seams still in app
- `scene_runtime::load_scene_document_for_mod`
- `scene_runtime::queue_scene_document_hydration`
- `scene_runtime::apply_scene_command`
- `scene_runtime::clear_runtime_scene_content_with_runtime`
- `script_runtime::dispatch_script_command_with_runtime`
- `orchestration::stabilize_runtime`
- `orchestration::process_placeholder_bridges`
- `scene_runtime::handlers::*`
- `script_runtime::handlers::*`
- `systems::*`
- `render_runtime::*`

## Dev console / debug boundary
- `amigo-app` owns the console shell, overlay presentation, and host-facing controls.
- Domain-owned console commands remain temporary app seams until P0.2 domain contributions.
- Debug overlay ownership stays app-side and renders after post-fx in the hosted render path.
