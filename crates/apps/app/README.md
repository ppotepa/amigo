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
- Prefer `run_hosted_with_options` for hosted execution.
- The raw bootstrap implementation remains crate-internal as a migration seam.

## P0.1 closure status
- `RuntimeSession` is the central lifecycle boundary used by hosted app flow.
- Startup scene load and queue go through `load_scene_document_for_session` and `queue_scene_document_hydration_for_session`.
- Startup orchestration routes scene commands through `apply_scene_command_for_session`.
- Render lifecycle goes through `build_render_frame_for_session` and updates `RenderSessionService` through `RuntimeSession`.
- System phases go through `run_app_system_phase_for_session` and update `SchedulerSessionService`.
- Script dispatch goes through `dispatch_script_command_for_session` and updates `ScriptSessionService`.
- Remaining app-owned scene/script/systems/render helpers are explicit temporary migration seams (old adapters) until domain providers replace them.

## Migration seams still in app
- `register_app_dev_console_command_provider`
- `register_app_script_command_provider`
- `register_app_scene_command_provider`
- `register_app_system_provider`
- `register_app_render_extractor_provider`
- `register_app_diagnostics_provider` / `register_app_metadata_provider`
- `scene_runtime::handlers::*` (wrapped through `App*` providers)
- `script_runtime::handlers::*` (wrapped through `App*` providers)
- `systems::*` (wrapped through `AppSystemsProvider`)
- `render_runtime::*` (wrapped through `AppRenderExtractorProvider`)

## Dev console / debug boundary
- `amigo-app` owns the console shell, overlay presentation, and host-facing controls.
- Domain-owned console commands are exposed via contribution descriptors; runtime uses
  temporary old providers in app until domain crates own them directly.
- Debug overlay ownership stays app-side and renders after post-fx in the hosted render path.
## Thin App Host boundary

`amigo-app` is a host, not the owner of runtime domain logic.

It owns:
- window/event loop
- WGPU surface and frame presentation
- host input bridge
- startup UX
- dev-console shell
- debug overlay presentation
- RuntimeSession wiring
- temporary adapters where a shared registry still requires host context

It must not own:
- domain scene command execution
- domain render extraction
- domain systems
- domain script command execution
- domain dev-console command logic
- domain diagnostics or metadata ownership

Runtime Capabilities describe valid installed capabilities only.
A capability is either domain-owned or `app.host`.
Domain code still physically in app is a migration blocker, not an `app.host` capability.
