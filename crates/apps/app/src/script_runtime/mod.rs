use super::*;
use std::sync::Arc;
use amigo_runtime::{HandlerDispatcher, HandlerRegistry, RoutedHandler};

struct AppScriptCommandContext<'a> {
    scene_command_queue: &'a SceneCommandQueue,
    script_event_queue: &'a ScriptEventQueue,
    dev_console_state: &'a DevConsoleState,
    asset_catalog: &'a AssetCatalog,
    ui_state_service: &'a UiStateService,
    audio_command_queue: &'a AudioCommandQueue,
    audio_scene_service: &'a AudioSceneService,
    diagnostics: &'a RuntimeDiagnostics,
    launch_selection: &'a LaunchSelection,
}

trait ScriptCommandHandler: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, command: &ScriptCommand) -> bool;
    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand);
}

type ScriptCommandHandlerObject =
    dyn for<'a> RoutedHandler<AppScriptCommandContext<'a>, ScriptCommand, ()>;

type ScriptCommandHandlerRegistry = HandlerRegistry<ScriptCommandHandlerObject>;

struct ScriptCommandHandlerAdapter<H>(H);

impl<H> RoutedHandler<AppScriptCommandContext<'_>, ScriptCommand, ()>
    for ScriptCommandHandlerAdapter<H>
where
    H: ScriptCommandHandler,
{
    fn name(&self) -> &'static str {
        self.0.name()
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        self.0.can_handle(command)
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        self.0.handle(ctx, command)
    }
}

fn register_script_command_handler<H>(
    registry: &mut ScriptCommandHandlerRegistry,
    handler: H,
) where
    H: ScriptCommandHandler + 'static,
{
    registry.register_arc(Arc::new(ScriptCommandHandlerAdapter(handler)));
}

pub(crate) struct ScriptCommandRuntimePlugin;

impl RuntimePlugin for ScriptCommandRuntimePlugin {
    fn name(&self) -> &'static str {
        "amigo-app-script-command-registry"
    }

    fn register(&self, services: &mut ServiceRegistry) -> AmigoResult<()> {
        let registry = build_script_command_registry();
        services.register(registry)
    }
}

struct SceneScriptCommandHandler;
struct RenderScriptCommandHandler;
struct AssetScriptCommandHandler;
struct AudioScriptCommandHandler;
struct UiScriptCommandHandler;
struct DebugScriptCommandHandler;

fn build_script_command_registry() -> ScriptCommandHandlerRegistry {
    let mut registry = ScriptCommandHandlerRegistry::new();
    register_script_command_handler(&mut registry, SceneScriptCommandHandler);
    register_script_command_handler(&mut registry, RenderScriptCommandHandler);
    register_script_command_handler(&mut registry, AssetScriptCommandHandler);
    register_script_command_handler(&mut registry, AudioScriptCommandHandler);
    register_script_command_handler(&mut registry, UiScriptCommandHandler);
    register_script_command_handler(&mut registry, DebugScriptCommandHandler);
    registry
}

fn dispatch_with_registry(
    registry: Arc<ScriptCommandHandlerRegistry>,
    command: ScriptCommand,
    scene_command_queue: &SceneCommandQueue,
    script_event_queue: &ScriptEventQueue,
    dev_console_state: &DevConsoleState,
    asset_catalog: &AssetCatalog,
    ui_state_service: &UiStateService,
    audio_command_queue: &AudioCommandQueue,
    audio_scene_service: &AudioSceneService,
    diagnostics: &RuntimeDiagnostics,
    launch_selection: &LaunchSelection,
) {
    let ctx = AppScriptCommandContext {
        scene_command_queue,
        script_event_queue,
        dev_console_state,
        asset_catalog,
        ui_state_service,
        audio_command_queue,
        audio_scene_service,
        diagnostics,
        launch_selection,
    };

    if HandlerDispatcher::new(registry).dispatch_first(|handler| {
        handler
            .can_handle(&command)
            .then(|| handler.handle(&ctx, command.clone()))
    })
    .is_none()
    {
        ctx.dev_console_state.write_line(format!(
            "unhandled placeholder script command: {}",
            crate::app_helpers::format_script_command(&command)
        ));
    }
}

pub(crate) fn dispatch_script_command_with_runtime(
    runtime: &Runtime,
    command: ScriptCommand,
) {
    let scene_command_queue = match required::<SceneCommandQueue>(runtime) {
        Ok(service) => service,
        Err(error) => {
            if let Ok(dev_console_state) = required::<DevConsoleState>(runtime) {
                dev_console_state.write_line(error.to_string());
            }
            return;
        }
    };
    let script_event_queue = match required::<ScriptEventQueue>(runtime) {
        Ok(service) => service,
        Err(error) => {
            if let Ok(dev_console_state) = required::<DevConsoleState>(runtime) {
                dev_console_state.write_line(error.to_string());
            }
            return;
        }
    };
    let dev_console_state = match required::<DevConsoleState>(runtime) {
        Ok(service) => service,
        Err(_) => return,
    };

    let Some(registry) = runtime.resolve::<ScriptCommandHandlerRegistry>() else {
        dev_console_state.write_line(
            "script command registry service is missing".to_owned(),
        );
        return;
    };

    let asset_catalog = match required::<AssetCatalog>(runtime) {
        Ok(service) => service,
        Err(error) => {
            dev_console_state.write_line(error.to_string());
            return;
        }
    };
    let ui_state_service = match required::<UiStateService>(runtime) {
        Ok(service) => service,
        Err(error) => {
            dev_console_state.write_line(error.to_string());
            return;
        }
    };
    let audio_command_queue = match required::<AudioCommandQueue>(runtime) {
        Ok(service) => service,
        Err(error) => {
            dev_console_state.write_line(error.to_string());
            return;
        }
    };
    let audio_scene_service = match required::<AudioSceneService>(runtime) {
        Ok(service) => service,
        Err(error) => {
            dev_console_state.write_line(error.to_string());
            return;
        }
    };
    let diagnostics = match required::<RuntimeDiagnostics>(runtime) {
        Ok(service) => service,
        Err(error) => {
            dev_console_state.write_line(error.to_string());
            return;
        }
    };
    let launch_selection = match required::<LaunchSelection>(runtime) {
        Ok(service) => service,
        Err(error) => {
            dev_console_state.write_line(error.to_string());
            return;
        }
    };

    dispatch_with_registry(
        registry,
        command,
        scene_command_queue.as_ref(),
        script_event_queue.as_ref(),
        dev_console_state.as_ref(),
        asset_catalog.as_ref(),
        ui_state_service.as_ref(),
        audio_command_queue.as_ref(),
        audio_scene_service.as_ref(),
        diagnostics.as_ref(),
        launch_selection.as_ref(),
    );
}

#[cfg(test)]
pub(crate) fn dispatch_script_command(
    command: ScriptCommand,
    scene_command_queue: &SceneCommandQueue,
    script_event_queue: &ScriptEventQueue,
    dev_console_state: &DevConsoleState,
    asset_catalog: &AssetCatalog,
    ui_state_service: &UiStateService,
    audio_command_queue: &AudioCommandQueue,
    audio_scene_service: &AudioSceneService,
    diagnostics: &RuntimeDiagnostics,
    launch_selection: &LaunchSelection,
) {
    dispatch_with_registry(
        Arc::new(build_script_command_registry()),
        command,
        scene_command_queue,
        script_event_queue,
        dev_console_state,
        asset_catalog,
        ui_state_service,
        audio_command_queue,
        audio_scene_service,
        diagnostics,
        launch_selection,
    );
}

impl ScriptCommandHandler for SceneScriptCommandHandler {
    fn name(&self) -> &'static str {
        "scene"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(command.namespace.as_str(), "scene")
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        match (command.name.as_str(), command.arguments.as_slice()) {
            ("select", [scene_id]) => {
                ctx.scene_command_queue.submit(SceneCommand::SelectScene {
                    scene: SceneKey::new(scene_id.clone()),
                });
            }
            ("reload", []) => {
                ctx.scene_command_queue.submit(SceneCommand::ReloadActiveScene);
            }
            ("spawn", [entity_name]) => {
                ctx.scene_command_queue.submit(SceneCommand::SpawnNamedEntity {
                    name: entity_name.clone(),
                    transform: None,
                });
            }
            ("clear", []) => {
                ctx.scene_command_queue.submit(SceneCommand::ClearEntities);
            }
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
    }
}

impl ScriptCommandHandler for RenderScriptCommandHandler {
    fn name(&self) -> &'static str {
        "render"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(
            command.namespace.as_str(),
            "2d.sprite" | "2d.text" | "3d.mesh" | "3d.material" | "3d.text"
        )
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        match (
            command.namespace.as_str(),
            command.name.as_str(),
            command.arguments.as_slice(),
        ) {
            ("2d.sprite", "spawn", [source_mod, entity_name, texture_key, width, height]) => {
                match crate::app_helpers::parse_scene_vec2(width, height, "2d sprite size") {
                    Ok(size) => ctx.scene_command_queue.submit(SceneCommand::QueueSprite2d {
                        command: Sprite2dSceneCommand::new(
                            source_mod.clone(),
                            entity_name.clone(),
                            AssetKey::new(texture_key.clone()),
                            size,
                        ),
                    }),
                    Err(message) => ctx.dev_console_state.write_line(message),
                }
            }
            ("2d.sprite", "spawn", [entity_name, texture_key, width, height]) => {
                match crate::app_helpers::parse_scene_vec2(width, height, "2d sprite size") {
                    Ok(size) => ctx.scene_command_queue.submit(SceneCommand::QueueSprite2d {
                        command: Sprite2dSceneCommand::new(
                            ctx.launch_selection.selected_mod(),
                            entity_name.clone(),
                            AssetKey::new(texture_key.clone()),
                            size,
                        ),
                    }),
                    Err(message) => ctx.dev_console_state.write_line(message),
                }
            }
            ("2d.text", "spawn", [source_mod, entity_name, content, font_key, width, height]) => {
                match crate::app_helpers::parse_scene_vec2(width, height, "2d text bounds") {
                    Ok(bounds) => ctx.scene_command_queue.submit(SceneCommand::QueueText2d {
                        command: Text2dSceneCommand::new(
                            source_mod.clone(),
                            entity_name.clone(),
                            content.clone(),
                            AssetKey::new(font_key.clone()),
                            bounds,
                        ),
                    }),
                    Err(message) => ctx.dev_console_state.write_line(message),
                }
            }
            ("2d.text", "spawn", [entity_name, content, font_key, width, height]) => {
                match crate::app_helpers::parse_scene_vec2(width, height, "2d text bounds") {
                    Ok(bounds) => ctx.scene_command_queue.submit(SceneCommand::QueueText2d {
                        command: Text2dSceneCommand::new(
                            ctx.launch_selection.selected_mod(),
                            entity_name.clone(),
                            content.clone(),
                            AssetKey::new(font_key.clone()),
                            bounds,
                        ),
                    }),
                    Err(message) => ctx.dev_console_state.write_line(message),
                }
            }
            ("3d.mesh", "spawn", [source_mod, entity_name, mesh_key]) => {
                ctx.scene_command_queue.submit(SceneCommand::QueueMesh3d {
                    command: Mesh3dSceneCommand::new(
                        source_mod.clone(),
                        entity_name.clone(),
                        AssetKey::new(mesh_key.clone()),
                    ),
                });
            }
            ("3d.mesh", "spawn", [entity_name, mesh_key]) => {
                ctx.scene_command_queue.submit(SceneCommand::QueueMesh3d {
                    command: Mesh3dSceneCommand::new(
                        ctx.launch_selection.selected_mod(),
                        entity_name.clone(),
                        AssetKey::new(mesh_key.clone()),
                    ),
                });
            }
            ("3d.material", "bind", [source_mod, entity_name, label, material_key]) => {
                ctx.scene_command_queue
                    .submit(SceneCommand::QueueMaterial3d {
                        command: Material3dSceneCommand::new(
                            source_mod.clone(),
                            entity_name.clone(),
                            label.clone(),
                            Some(AssetKey::new(material_key.clone())),
                        ),
                    });
            }
            ("3d.material", "bind", [entity_name, label, material_key]) => {
                ctx.scene_command_queue
                    .submit(SceneCommand::QueueMaterial3d {
                        command: Material3dSceneCommand::new(
                            ctx.launch_selection.selected_mod(),
                            entity_name.clone(),
                            label.clone(),
                            Some(AssetKey::new(material_key.clone())),
                        ),
                    });
            }
            ("3d.text", "spawn", [source_mod, entity_name, content, font_key, size]) => {
                match size.parse::<f32>() {
                    Ok(size) => ctx.scene_command_queue.submit(SceneCommand::QueueText3d {
                        command: Text3dSceneCommand::new(
                            source_mod.clone(),
                            entity_name.clone(),
                            content.clone(),
                            AssetKey::new(font_key.clone()),
                            size,
                        ),
                    }),
                    Err(error) => ctx.dev_console_state.write_line(format!(
                        "failed to parse 3d text size `{size}` as f32: {error}"
                    )),
                }
            }
            ("3d.text", "spawn", [entity_name, content, font_key, size]) => {
                match size.parse::<f32>() {
                    Ok(size) => ctx.scene_command_queue.submit(SceneCommand::QueueText3d {
                        command: Text3dSceneCommand::new(
                            ctx.launch_selection.selected_mod(),
                            entity_name.clone(),
                            content.clone(),
                            AssetKey::new(font_key.clone()),
                            size,
                        ),
                    }),
                    Err(error) => ctx.dev_console_state.write_line(format!(
                        "failed to parse 3d text size `{size}` as f32: {error}"
                    )),
                }
            }
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
    }
}

impl ScriptCommandHandler for AssetScriptCommandHandler {
    fn name(&self) -> &'static str {
        "asset"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(command.namespace.as_str(), "asset")
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        match (command.name.as_str(), command.arguments.as_slice()) {
            ("reload", [asset_key]) => {
                crate::orchestration::request_asset_reload(
                    ctx.asset_catalog,
                    asset_key,
                    AssetLoadPriority::Immediate,
                    ctx.dev_console_state,
                );
                ctx.script_event_queue.publish(ScriptEvent::new(
                    "asset.reload-requested",
                    vec![asset_key.clone()],
                ));
            }
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
    }
}

impl ScriptCommandHandler for AudioScriptCommandHandler {
    fn name(&self) -> &'static str {
        "audio"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(command.namespace.as_str(), "audio")
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        match (command.name.as_str(), command.arguments.as_slice()) {
            ("preload", [clip_name]) => {
                let asset_key =
                    crate::app_helpers::resolve_mod_audio_asset_key(ctx.launch_selection, clip_name);
                crate::app_helpers::register_audio_clip_reference(
                    ctx.asset_catalog,
                    ctx.audio_scene_service,
                    &asset_key,
                    AudioPlaybackMode::OneShot,
                );
                ctx.dev_console_state
                    .write_line(format!("preloaded audio clip `{}`", asset_key.as_str()));
            }
            ("play", [clip_name]) => {
                let asset_key = crate::app_helpers::resolve_mod_audio_asset_key(ctx.launch_selection, clip_name);
                crate::app_helpers::register_audio_clip_reference(
                    ctx.asset_catalog,
                    ctx.audio_scene_service,
                    &asset_key,
                    AudioPlaybackMode::OneShot,
                );
                ctx.audio_command_queue.push(AudioCommand::PlayOnce {
                    clip: AudioClipKey::new(asset_key.as_str().to_owned()),
                });
                ctx.dev_console_state
                    .write_line(format!("queued audio one-shot `{}`", asset_key.as_str()));
            }
            ("play-asset", [asset_key]) => {
                let asset_key = AssetKey::new(asset_key.clone());
                crate::app_helpers::register_audio_clip_reference(
                    ctx.asset_catalog,
                    ctx.audio_scene_service,
                    &asset_key,
                    AudioPlaybackMode::OneShot,
                );
                ctx.audio_command_queue.push(AudioCommand::PlayOnce {
                    clip: AudioClipKey::new(asset_key.as_str().to_owned()),
                });
                ctx.dev_console_state
                    .write_line(format!("queued audio one-shot `{}`", asset_key.as_str()));
            }
            ("start-realtime", [source]) => {
                let asset_key = crate::app_helpers::resolve_mod_audio_asset_key(ctx.launch_selection, source);
                crate::app_helpers::register_audio_clip_reference(
                    ctx.asset_catalog,
                    ctx.audio_scene_service,
                    &asset_key,
                    AudioPlaybackMode::Looping,
                );
                ctx.audio_command_queue.push(AudioCommand::StartSource {
                    source: AudioSourceId::new(source.clone()),
                    clip: AudioClipKey::new(asset_key.as_str().to_owned()),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued realtime audio source `{}` using `{}`",
                    source,
                    asset_key.as_str()
                ));
            }
            ("stop", [source]) => {
                ctx.audio_command_queue.push(AudioCommand::StopSource {
                    source: AudioSourceId::new(source.clone()),
                });
                ctx.dev_console_state
                    .write_line(format!("queued stop for audio source `{source}`"));
            }
            ("set-param", [source, param, value]) => match value.parse::<f32>() {
                Ok(value) => {
                    ctx.audio_command_queue.push(AudioCommand::SetParam {
                        source: AudioSourceId::new(source.clone()),
                        param: param.clone(),
                        value,
                    });
                }
                Err(error) => ctx.dev_console_state.write_line(format!(
                    "failed to parse audio param value `{value}` as f32: {error}"
                )),
            },
            ("set-volume", [bus, value]) => match value.parse::<f32>() {
                Ok(value) if bus == "master" => {
                    ctx.audio_command_queue
                        .push(AudioCommand::SetMasterVolume { value });
                    ctx.dev_console_state.write_line(format!(
                        "queued master audio volume = {}",
                        value.clamp(0.0, 1.0)
                    ));
                }
                Ok(value) => {
                    ctx.audio_command_queue.push(AudioCommand::SetVolume {
                        bus: bus.clone(),
                        value,
                    });
                    ctx.dev_console_state.write_line(format!(
                        "queued audio bus volume `{bus}` = {}",
                        value.clamp(0.0, 1.0)
                    ));
                }
                Err(error) => ctx.dev_console_state.write_line(format!(
                    "failed to parse audio volume `{value}` as f32: {error}"
                )),
            },
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
    }
}

impl ScriptCommandHandler for UiScriptCommandHandler {
    fn name(&self) -> &'static str {
        "ui"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(command.namespace.as_str(), "ui")
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        match (command.name.as_str(), command.arguments.as_slice()) {
            ("set-text", [path, value]) => {
                if ctx.ui_state_service.set_text(path.clone(), value.clone()) {
                    ctx.dev_console_state
                        .write_line(format!("updated ui text override `{path}`"));
                }
            }
            ("set-value", [path, value]) => match value.parse::<f32>() {
                Ok(value) => {
                    if ctx.ui_state_service.set_value(path.clone(), value) {
                        ctx.dev_console_state.write_line(format!(
                            "updated ui value override `{path}` to {}",
                            value.clamp(0.0, 1.0)
                        ));
                    }
                }
                Err(error) => ctx.dev_console_state.write_line(format!(
                    "failed to parse ui value `{value}` as f32: {error}"
                )),
            },
            ("show", [path]) => {
                if ctx.ui_state_service.show(path.clone()) {
                    ctx.dev_console_state
                        .write_line(format!("showed ui path `{path}`"));
                }
            }
            ("hide", [path]) => {
                if ctx.ui_state_service.hide(path.clone()) {
                    ctx.dev_console_state.write_line(format!("hid ui path `{path}`"));
                }
            }
            ("enable", [path]) => {
                if ctx.ui_state_service.enable(path.clone()) {
                    ctx.dev_console_state
                        .write_line(format!("enabled ui path `{path}`"));
                }
            }
            ("disable", [path]) => {
                if ctx.ui_state_service.disable(path.clone()) {
                    ctx.dev_console_state
                        .write_line(format!("disabled ui path `{path}`"));
                }
            }
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
    }
}

impl ScriptCommandHandler for DebugScriptCommandHandler {
    fn name(&self) -> &'static str {
        "debug"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(command.namespace.as_str(), "debug" | "dev-shell")
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        match (
            command.namespace.as_str(),
            command.name.as_str(),
            command.arguments.as_slice(),
        ) {
            ("debug", "log", [line]) => {
                ctx.dev_console_state.write_line(format!("script: {line}"));
            }
            ("debug", "warn", [line]) => {
                ctx.dev_console_state
                    .write_line(format!("script warning: {line}"));
            }
            ("dev-shell", "refresh-diagnostics", [target_mod]) => {
                ctx.dev_console_state.write_line(format!(
                    "diagnostics refreshed for mod={} scene={} window={} input={} render={} script={}",
                    target_mod,
                    ctx.launch_selection.selected_scene(),
                    ctx.diagnostics.window_backend,
                    ctx.diagnostics.input_backend,
                    ctx.diagnostics.render_backend,
                    ctx.diagnostics.script_backend
                ));
                ctx.script_event_queue.publish(ScriptEvent::new(
                    "dev-shell.diagnostics-refreshed",
                    vec![target_mod.clone()],
                ));
            }
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
    }
}
