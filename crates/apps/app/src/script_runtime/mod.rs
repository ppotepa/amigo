//! App-side scripting runtime integration.
//! This module wires script events and commands between the Rhai backend and the live runtime.

use super::*;
use amigo_runtime::{HandlerDispatcher, HandlerRegistry};
use amigo_session::{
    runtime_capabilities::{
        RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeCapability,
        RuntimeDomainId, ScriptCommandHandlerContribution, ScriptCommandHandlerDescriptor,
        ScriptCommandProvider, APP_HOST_DOMAIN_ID,
    },
    ScriptCommandHandler,
    RuntimeSession,
};
use std::sync::Arc;

mod handlers;

pub(super) struct AppScriptCommandContext<'a> {
    scene_command_queue: &'a SceneCommandQueue,
    script_event_queue: &'a ScriptEventQueue,
    dev_console_state: &'a DevConsoleState,
    asset_catalog: &'a AssetCatalog,
    audio_command_queue: &'a AudioCommandQueue,
    audio_scene_service: &'a AudioSceneService,
    diagnostics: &'a RuntimeDiagnostics,
    launch_selection: &'a LaunchSelection,
}

type ScriptCommandHandlerObject =
    dyn for<'a> ScriptCommandHandler<AppScriptCommandContext<'a>, ScriptCommand, ()>;

pub(super) type ScriptCommandHandlerRegistry = HandlerRegistry<ScriptCommandHandlerObject>;

pub(super) fn register_script_command_handler<H>(
    registry: &mut ScriptCommandHandlerRegistry,
    handler: H,
) where
    H: for<'a> ScriptCommandHandler<AppScriptCommandContext<'a>, ScriptCommand, ()>
        + 'static,
{
    registry.register_arc(Arc::new(handler));
}

pub(crate) struct ScriptCommandRuntimePlugin;
struct AppHostScriptRuntimeHandler;

pub(crate) struct HostAppScriptCommandProvider;

impl ScriptCommandProvider for HostAppScriptCommandProvider {
    fn register_script_command_handlers(
        &self,
        descriptors: &mut Vec<ScriptCommandHandlerDescriptor>,
    ) {
        descriptors.extend(
            ["debug", "dev-shell"]
            .into_iter()
            .map(|handler_id| ScriptCommandHandlerDescriptor {
                descriptor: RuntimeCapabilityDescriptor {
                    domain_id: RuntimeDomainId::new(APP_HOST_DOMAIN_ID),
                    kind: RuntimeCapabilityKind::ScriptCommandHandler,
                    id: format!("{handler_id}.script"),
                    label: handler_id.to_string(),
                    description: "app host script command handler".to_string(),
                    capabilities: Vec::new(),
                    tags: vec!["app".to_string(), "host".to_string()],
                    migration_seam: false,
                },
                handler_id: handler_id.to_string(),
            }),
        );
    }
}

pub(crate) fn register_host_script_command_provider(
    session: &mut RuntimeSession,
) -> Vec<ScriptCommandHandlerContribution> {
    let mut descriptors = Vec::new();
    HostAppScriptCommandProvider.register_script_command_handlers(&mut descriptors);
    let contributions = descriptors
        .into_iter()
        .map(|descriptor| ScriptCommandHandlerContribution {
            descriptor: descriptor.clone(),
        })
        .collect::<Vec<_>>();

    for contribution in &contributions {
        session
            .runtime_capabilities_mut()
            .register(RuntimeCapability {
                descriptor: contribution.descriptor.descriptor.clone(),
            });
    }

    contributions
}

impl RuntimePlugin for ScriptCommandRuntimePlugin {
    fn name(&self) -> &'static str {
        "amigo-app-script-command-registry"
    }

    fn register(&self, services: &mut ServiceRegistry) -> AmigoResult<()> {
        services.register(amigo_scripting_api::RuntimeScriptCommandHandlerRegistry::new())?;
        let runtime_handlers =
            services.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
        amigo_scripting_api::register_runtime_script_command_handler(
            runtime_handlers.as_ref(),
            AppHostScriptRuntimeHandler,
        );
        Ok(())
    }
}

impl amigo_scripting_api::RuntimeScriptCommandHandler for AppHostScriptRuntimeHandler {
    fn name(&self) -> &'static str {
        "app-host-script-adapter"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        if matches!(command.namespace.as_str(), "debug" | "dev-shell") {
            return true;
        }
        if command.namespace == "audio" {
            return matches!(
                (command.name.as_str(), command.arguments.len()),
                ("preload", 1) | ("play", 1) | ("start-realtime", 1)
            );
        }
        matches!(
            (
                command.namespace.as_str(),
                command.name.as_str(),
                command.arguments.len(),
            ),
            ("2d.sprite", "spawn", 4)
                | ("2d.text", "spawn", 5)
                | ("3d.mesh", "spawn", 2)
                | ("3d.material", "bind", 3)
                | ("3d.text", "spawn", 4)
        )
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let scene_command_queue = required::<SceneCommandQueue>(runtime)?;
        let script_event_queue = required::<ScriptEventQueue>(runtime)?;
        let dev_console_state = required::<DevConsoleState>(runtime)?;
        let asset_catalog = required::<AssetCatalog>(runtime)?;
        let audio_command_queue = required::<AudioCommandQueue>(runtime)?;
        let audio_scene_service = required::<AudioSceneService>(runtime)?;
        let diagnostics = required::<RuntimeDiagnostics>(runtime)?;
        let launch_selection = required::<LaunchSelection>(runtime)?;

        dispatch_with_registry(
            Arc::new(build_script_command_registry()),
            command,
            scene_command_queue.as_ref(),
            script_event_queue.as_ref(),
            dev_console_state.as_ref(),
            asset_catalog.as_ref(),
            audio_command_queue.as_ref(),
            audio_scene_service.as_ref(),
            diagnostics.as_ref(),
            launch_selection.as_ref(),
        );
        Ok(())
    }
}

fn build_script_command_registry() -> ScriptCommandHandlerRegistry {
    let mut registry = ScriptCommandHandlerRegistry::new();
    handlers::register_builtin_script_command_handlers(&mut registry);
    registry
}

fn dispatch_with_registry(
    registry: Arc<ScriptCommandHandlerRegistry>,
    command: ScriptCommand,
    scene_command_queue: &SceneCommandQueue,
    script_event_queue: &ScriptEventQueue,
    dev_console_state: &DevConsoleState,
    asset_catalog: &AssetCatalog,
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
        audio_command_queue,
        audio_scene_service,
        diagnostics,
        launch_selection,
    };

    if HandlerDispatcher::new(registry)
        .dispatch_first(|handler| {
            handler
                .can_handle(&command)
                .then(|| handler.handle(&ctx, command.clone()))
        })
        .is_none()
    {
        ctx.dev_console_state.write_line(format!(
            "unhandled script command: {}",
            crate::app_helpers::format_script_command(&command)
        ));
    }
}

pub(crate) fn dispatch_script_command_with_runtime(
    runtime: &Runtime,
    command: ScriptCommand,
) -> AmigoResult<()> {
    let dev_console_state = match required::<DevConsoleState>(runtime) {
        Ok(service) => service,
        Err(error) => return Err(error),
    };

    let registry = runtime.required::<amigo_scripting_api::RuntimeScriptCommandHandlerRegistry>()?;
    let result = amigo_runtime::HandlerDispatcher::new(registry).dispatch_first(|handler| {
        handler
            .can_handle(&command)
            .then(|| handler.handle(runtime, command.clone()))
    });
    if let Some(result) = result {
        result?;
    } else {
        dev_console_state.write_line(format!(
            "unhandled script command: {}",
            crate::app_helpers::format_script_command(&command)
        ));
    }
    Ok(())
}

pub(crate) fn dispatch_script_command_for_session(
    session: &RuntimeSession,
    command: ScriptCommand,
) -> AmigoResult<()> {
    let command_label = crate::app_helpers::format_script_command(&command);
    session.begin_script_command_dispatch(command_label.clone());
    if let Err(error) = dispatch_script_command_with_runtime(session.runtime(), command) {
        session.mark_script_dispatch_error(command_label, error.to_string());
        return Err(error);
    }

    session.complete_script_command_dispatch();
    Ok(())
}

#[cfg(test)]
pub(crate) fn dispatch_script_command(
    command: ScriptCommand,
    scene_command_queue: &SceneCommandQueue,
    script_event_queue: &ScriptEventQueue,
    dev_console_state: &DevConsoleState,
    asset_catalog: &AssetCatalog,
    _ui_state_service: &UiStateService,
    audio_command_queue: &AudioCommandQueue,
    audio_scene_service: &AudioSceneService,
    diagnostics: &RuntimeDiagnostics,
    launch_selection: &LaunchSelection,
) {
    let layered_image_scene_service = amigo_2d_layered_image::LayeredImageSceneService::default();
    let render_layer2d_scene_service = amigo_2d_composition::RenderLayer2dSceneService::default();
    let global_light2d_scene_service = amigo_2d_lighting::GlobalLight2dSceneService::default();
    let light_group2d_scene_service = amigo_2d_lighting::LightGroup2dSceneService::default();
    dispatch_script_command_with_layered_image_service(
        command,
        scene_command_queue,
        script_event_queue,
        dev_console_state,
        asset_catalog,
        &layered_image_scene_service,
        &render_layer2d_scene_service,
        &global_light2d_scene_service,
        &light_group2d_scene_service,
        _ui_state_service,
        audio_command_queue,
        audio_scene_service,
        diagnostics,
        launch_selection,
    );
}

#[cfg(test)]
pub(crate) fn dispatch_script_command_with_layered_image_service(
    command: ScriptCommand,
    scene_command_queue: &SceneCommandQueue,
    script_event_queue: &ScriptEventQueue,
    dev_console_state: &DevConsoleState,
    asset_catalog: &AssetCatalog,
    _layered_image_scene_service: &amigo_2d_layered_image::LayeredImageSceneService,
    _render_layer2d_scene_service: &amigo_2d_composition::RenderLayer2dSceneService,
    _global_light2d_scene_service: &amigo_2d_lighting::GlobalLight2dSceneService,
    _light_group2d_scene_service: &amigo_2d_lighting::LightGroup2dSceneService,
    _ui_state_service: &UiStateService,
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
        audio_command_queue,
        audio_scene_service,
        diagnostics,
        launch_selection,
    );
}
