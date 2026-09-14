use crate::{render::NprPlaygroundRenderService, state::NprPlaygroundState};
use amigo_capabilities::{DEFAULT_CAPABILITY_VERSION, register_domain_plugin};
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
#[derive(Default)]
struct Lifecycle {
    scene: Mutex<Option<String>>,
    npr_active: Mutex<bool>,
    mouse: Mutex<Option<(f32, f32)>>,
    zoom: Mutex<crate::zoom::SmoothZoom>,
    zoom_center: Mutex<Option<glam::Vec3>>,
    dynamic_ready: Mutex<bool>,
}
pub struct NprPlugin;
impl RuntimePlugin for NprPlugin {
    fn name(&self) -> &'static str {
        "amigo-npr-plugin"
    }
    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(NprPlaygroundState::default())?;
        registry.register(crate::playground::NprPlaygroundService::new(
            registry.required::<NprPlaygroundState>()?,
        ))?;
        let playground = registry.required::<crate::playground::NprPlaygroundService>()?;
        if let Some(host) = registry.resolve::<amigo_playground_api::PlaygroundHostService>() {
            host.register(playground)
                .map_err(amigo_core::AmigoError::Message)?;
        }
        registry.register(NprPlaygroundRenderService::default())?;
        registry
            .required::<crate::playground::NprPlaygroundService>()?
            .attach_runtime(
                registry.required::<amigo_assets::AssetCatalog>()?,
                registry.required::<NprPlaygroundRenderService>()?,
            );
        registry.register(Lifecycle::default())?;
        if !registry.has::<amigo_editor_ingame::IngameEditorRuntimeApplyProviderRegistry>() {
            registry.register(
                amigo_editor_ingame::IngameEditorRuntimeApplyProviderRegistry::default(),
            )?;
        }
        if let Some(editor_apply) =
            registry.resolve::<amigo_editor_ingame::IngameEditorRuntimeApplyProviderRegistry>()
        {
            editor_apply.register(crate::NprPlaygroundEditorRuntimeApplyProvider);
        }
        amigo_scene::register_scene_component_plugin_spec::<
            crate::scene::NprPlaygroundSceneComponentSpec,
        >(registry)?;
        let scene_handlers =
            registry.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
        amigo_scene::register_runtime_scene_command_handler(
            scene_handlers.as_ref(),
            crate::scene::NprPlaygroundSceneCommandHandler,
        );
        if let Some(plugin_scene_handlers) =
            registry.resolve::<amigo_scene::ScenePluginCommandHandlerRegistry>()
        {
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
                            // A finished worker from the previous scene must
                            // never publish its packet into this scene. Dropping
                            // its receiver is a cheap cancellation boundary;
                            // the worker owns only immutable input snapshots.
                            *lifecycle.dynamic_ready.lock().unwrap() = false;
                            *lifecycle.zoom.lock().unwrap() = Default::default();
                            *lifecycle.mouse.lock().unwrap() = None;
                            let npr_active = if let Some(source) = mods.mod_by_id(&doc.source_mod) {
                                let scene_path = source.root_path.join(&doc.relative_path);
                                if scene_uses_npr(&scene_path)? {
                                    let service = runtime
                                        .required::<crate::playground::NprPlaygroundService>()?;
                                    amigo_playground_api::PlaygroundProvider::open_scene(
                                        service.as_ref(),
                                        &source.root_path,
                                        &scene_path,
                                    )
                                    .map_err(amigo_core::AmigoError::Message)?;
                                    let _ = state.take_staged_authored_scene();
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            };
                            *lifecycle.npr_active.lock().unwrap() = npr_active;
                            runtime
                                .required::<NprPlaygroundRenderService>()?
                                .set_scene_is_npr(npr_active);
                            *active = Some(key);
                        }
                    } else {
                        *lifecycle.npr_active.lock().unwrap() = false;
                        let render = runtime.required::<NprPlaygroundRenderService>()?;
                        render.set_scene_is_npr(false);
                        render.clear();
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
                runtime
                    .required::<crate::playground::NprPlaygroundService>()?
                    .advance_camera(amigo_session::host_delta_seconds(runtime));
                if !*runtime.required::<Lifecycle>()?.npr_active.lock().unwrap() {
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
                                // selected object. A hit on another source is
                                // ignored instead of mixing meshes.
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
                let active = *runtime.required::<Lifecycle>()?.npr_active.lock().unwrap();
                if !active {
                    *runtime
                        .required::<Lifecycle>()?
                        .dynamic_ready
                        .lock()
                        .unwrap() = false;
                    runtime.required::<NprPlaygroundRenderService>()?.clear();
                    return Ok(());
                }
                let state = runtime.required::<NprPlaygroundState>()?;
                state.record_frame();
                let viewport = runtime.required::<amigo_ui::UiInputViewportState>()?.get();
                if let Some(viewport) = viewport {
                    let viewport = [viewport.width as u32, viewport.height as u32];
                    *state.viewport.lock().unwrap() = viewport;
                    let lifecycle = runtime.required::<Lifecycle>()?;
                    let loading = runtime.resolve::<amigo_session::RuntimeLoadingService>();
                    let scene_service = runtime.required::<amigo_scene::SceneService>()?;
                    let mesh_scene_service =
                        runtime.required::<amigo_3d_mesh::MeshSceneService>()?;
                    let meshes = amigo_3d_mesh::extract_mesh3d_render_commands(
                        amigo_3d_mesh::Mesh3dRenderExtractionContext {
                            scene_service: scene_service.as_ref(),
                            mesh_scene_service: mesh_scene_service.as_ref(),
                        },
                    );
                    if meshes.is_empty() {
                        let render = runtime.required::<NprPlaygroundRenderService>()?;
                        let view = render.fork_view();
                        view.rebuild_with_delta(&state.render_snapshot(), viewport, 0.0)
                            .map_err(amigo_core::AmigoError::Message)?;
                        render.publish_scene_snapshot(view.commands(), view.background());
                    }
                    let missing_geometry = meshes
                        .iter()
                        .filter(|mesh| mesh.mesh.geometry.is_none())
                        .count();
                    if missing_geometry > 0 {
                        if let Some(loading) = loading.as_ref() {
                            loading.set_stage(
                                "Loading mesh geometry",
                                Some(format!(
                                    "{} mesh instances waiting for GLB import",
                                    missing_geometry
                                )),
                            );
                        }
                    } else {
                        let mut ready = lifecycle.dynamic_ready.lock().unwrap();
                        if !*ready {
                            if let Some(loading) = loading.as_ref() {
                                loading.add_work(1);
                                loading.set_stage(
                                    "Preparing dynamic NPR scene",
                                    Some(format!("{} mesh instances", meshes.len())),
                                );
                            }
                            *ready = true;
                        } else if let Some(loading) = loading.as_ref() {
                            if loading.snapshot().state
                                == amigo_session::RuntimeLoadingState::Loading
                            {
                                loading.complete_work(1);
                                loading.ready();
                            }
                        }
                    }

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

fn scene_uses_npr(path: &Path) -> Result<bool, amigo_core::AmigoError> {
    let document: serde_yaml::Value = serde_yaml::from_slice(
        &std::fs::read(path).map_err(|error| amigo_core::AmigoError::Message(error.to_string()))?,
    )
    .map_err(|error| amigo_core::AmigoError::Message(error.to_string()))?;
    Ok(document
        .get("entities")
        .and_then(serde_yaml::Value::as_sequence)
        .into_iter()
        .flatten()
        .filter_map(|entity| {
            entity
                .get("components")
                .and_then(serde_yaml::Value::as_sequence)
        })
        .flatten()
        .any(|component| {
            component.get("type").and_then(serde_yaml::Value::as_str)
                == Some(crate::scene::NPR_SETTINGS_COMPONENT_TYPE)
        }))
}
