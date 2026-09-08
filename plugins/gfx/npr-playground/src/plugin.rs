use crate::{render::NprPlaygroundRenderService, state::NprPlaygroundState};
use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};
use std::sync::{Arc, Mutex};
#[derive(Default)]
struct Lifecycle {
    scene: Mutex<Option<String>>,
    mouse: Mutex<Option<(f32, f32)>>,
    zoom: Mutex<crate::zoom::SmoothZoom>,
    zoom_center: Mutex<Option<glam::Vec3>>,
}
pub struct NprPlaygroundPlugin;
impl RuntimePlugin for NprPlaygroundPlugin {
    fn name(&self) -> &'static str {
        "amigo-npr-playground-plugin"
    }
    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(NprPlaygroundState::default())?;
        registry.register(NprPlaygroundRenderService::default())?;
        registry.register(Lifecycle::default())?;
        amigo_scene::register_scene_component_plugin_spec::<
            crate::scene::NprPlaygroundSceneComponentSpec,
        >(registry)?;
        let state = registry.required::<NprPlaygroundState>()?;
        registry
            .required::<amigo_runtime_control::RuntimeControlService>()?
            .register_provider(state.clone());
        registry
            .required::<amigo_panels::PresetService>()?
            .register(state.clone());
        registry
            .required::<amigo_panels::PresetService>()?
            .register(std::sync::Arc::new(
                crate::state::look_presets::LookPresetProvider(state),
            ));
        let scene_handlers = registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene::NprPlaygroundSceneCommandHandler,
        );
        if let Some(plugin_scene_handlers) = registry.resolve::<amigo_scene::ScenePluginCommandHandlerRegistry>() {
            plugin_scene_handlers.register(
                crate::scene::NPR_PLAYGROUND_SCENE_COMMAND_TYPE,
                Arc::new(crate::scene::NprPlaygroundSceneCommandHandler),
            );
        }
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::Update,
            "npr_playground_update",
            |runtime| {
                let state = runtime.required::<NprPlaygroundState>()?;
                let lifecycle = runtime.required::<Lifecycle>()?;
                if let (Some(session), Some(mods)) = (
                    runtime.resolve::<amigo_session::SceneSessionService>(),
                    runtime.resolve::<amigo_modding::ModCatalog>(),
                ) {
                    let snapshot = session.snapshot();
                    if let Some(doc) = snapshot.loaded_scene_document() {
                        let key = format!(
                            "{}:{}:{}",
                            doc.source_mod,
                            doc.scene_id,
                            snapshot.lifecycle_summary().clear_count
                        );
                        let mut active = lifecycle.scene.lock().unwrap();
                        if active.as_ref() != Some(&key) {
                            if !state
                                .apply_staged_authored_scene()
                                .map_err(amigo_core::AmigoError::Message)?
                            {
                                state.configure_scene(doc.scene_id == "gallery");
                            }
                            *lifecycle.zoom.lock().unwrap() = Default::default();
                            *lifecycle.mouse.lock().unwrap() = None;
                            if let Some(source) = mods.mod_by_id(&doc.source_mod) {
                                if doc.source_mod == "npr-playground" {
                                    runtime
                                        .required::<NprPlaygroundRenderService>()?
                                        .load_models(&source.root_path)
                                        .map_err(amigo_core::AmigoError::Message)?;
                                }
                            }
                            *active = Some(key);
                        } else {
                            // Hydration and Update can occur in different
                            // frames. A late command is still applied exactly
                            // once, without resetting live metadata edits.
                            state
                                .apply_staged_authored_scene()
                                .map_err(amigo_core::AmigoError::Message)?;
                        }
                    }
                }
                state.tick(amigo_session::simulation_delta_seconds(runtime));
                Ok(())
            },
        );
        // Update may run several simulation ticks per host frame. Consume viewport
        // input once, even while simulation is paused, without multiplying wheel deltas.
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::PostUpdate,
            "npr_playground_camera",
            |runtime| {
                if !runtime
                    .required::<amigo_session::SceneSessionService>()?
                    .snapshot()
                    .loaded_scene_document()
                    .is_some_and(|doc| doc.source_mod == "npr-playground")
                {
                    return Ok(());
                }
                let state = runtime.required::<NprPlaygroundState>()?;
                let lifecycle = runtime.required::<Lifecycle>()?;
                if let Some(input) = runtime.resolve::<amigo_ui::UiInputService>() {
                    let input = input.snapshot();
                    let mut last = lifecycle.mouse.lock().unwrap();
                    let mut settings = state.settings.lock().unwrap();
                    let authoring = state.construction_authoring_active();
                    if authoring
                        && state.construction_authoring_accepts_click(input.mouse_left_down)
                        && input.mouse_left_pressed
                    {
                        if let Some(point) = input.mouse_position {
                            let viewport = *state.viewport.lock().unwrap();
                            let selected = settings.selected.clone();
                            if let Some(pick) = runtime
                                .required::<NprPlaygroundRenderService>()?
                                .pick_surface(
                                    &settings,
                                    viewport,
                                    glam::Vec2::new(point.x, point.y),
                                )
                            {
                                // The tool intentionally stays bound to the
                                // selected object. A gallery hit on another
                                // model is ignored instead of mixing meshes.
                                if pick.object_id == selected {
                                    drop(settings);
                                    state
                                        .place_construction_anchor(&pick.object_id, pick.anchor)
                                        .map_err(amigo_core::AmigoError::Message)?;
                                    settings = state.settings.lock().unwrap();
                                }
                            }
                        }
                    } else if !authoring && input.mouse_left_down {
                        if let (Some(previous), Some(point)) = (*last, input.mouse_position) {
                            settings.camera_yaw += (point.x - previous.0) * 0.3;
                            settings.camera_pitch = (settings.camera_pitch
                                + (point.y - previous.1) * 0.3)
                                .clamp(-89.0, 89.0);
                        }
                    }
                    *last = input.mouse_position.map(|p| (p.x, p.y));
                    let mut zoom = lifecycle.zoom.lock().unwrap();
                    let mut center = lifecycle.zoom_center.lock().unwrap();
                    if *center != Some(settings.camera_target) {
                        *zoom = Default::default();
                        *center = Some(settings.camera_target);
                    }
                    settings.camera_distance = zoom.advance(
                        settings.camera_distance,
                        input.mouse_wheel_y,
                        amigo_session::host_delta_seconds(runtime),
                    );
                }
                Ok(())
            },
        );
        registry.required::<SystemRegistry>()?.register_fn(
            SystemPhase::RenderExtract,
            "npr_playground_extract",
            |runtime| {
                let active = runtime
                    .required::<amigo_session::SceneSessionService>()?
                    .snapshot()
                    .loaded_scene_document()
                    .is_some_and(|doc| doc.source_mod == "npr-playground");
                if !active {
                    runtime.required::<NprPlaygroundRenderService>()?.clear();
                    return Ok(());
                }
                runtime.required::<NprPlaygroundState>()?.record_frame();
                let viewport = runtime.required::<amigo_ui::UiInputViewportState>()?.get();
                if let Some(viewport) = viewport {
                    *runtime
                        .required::<NprPlaygroundState>()?
                        .viewport
                        .lock()
                        .unwrap() = [viewport.width as u32, viewport.height as u32];
                    let settings = runtime.required::<NprPlaygroundState>()?.render_snapshot();
                    runtime
                        .required::<NprPlaygroundRenderService>()?
                        .rebuild_with_delta(
                            &settings,
                            [viewport.width as u32, viewport.height as u32],
                            amigo_session::host_delta_seconds(runtime),
                        )
                        .map_err(amigo_core::AmigoError::Message)?;
                    let rendered = runtime.required::<NprPlaygroundRenderService>()?.stats();
                    for (key, value) in runtime
                        .required::<NprPlaygroundState>()?
                        .render_stats
                        .lock()
                        .unwrap()
                        .iter_mut()
                    {
                        *value = rendered.get(key).copied().unwrap_or(0);
                    }
                }
                Ok(())
            },
        );
        registry
            .required::<amigo_render_api::RuntimeRenderExtractorIdRegistry>()?
            .register(crate::render::NPR_PLAYGROUND_EXTRACTOR_ID);
        register_domain_plugin(
            registry,
            "amigo.gfx.npr-playground",
            &["gfx.npr@1"],
            &[],
            DEFAULT_CAPABILITY_VERSION,
        )?;
        Ok(())
    }
}
