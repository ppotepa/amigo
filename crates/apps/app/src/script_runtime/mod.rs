use super::*;
use amigo_runtime::{HandlerDispatcher, HandlerRegistry, RoutedHandler};
use std::sync::Arc;

mod handlers;

pub(super) struct AppScriptCommandContext<'a> {
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

pub(super) trait ScriptCommandHandler: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, command: &ScriptCommand) -> bool;
    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand);
}

type ScriptCommandHandlerObject =
    dyn for<'a> RoutedHandler<AppScriptCommandContext<'a>, ScriptCommand, ()>;

pub(super) type ScriptCommandHandlerRegistry = HandlerRegistry<ScriptCommandHandlerObject>;

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

pub(super) fn register_script_command_handler<H>(
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

    if HandlerDispatcher::new(registry)
        .dispatch_first(|handler| {
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

pub(crate) fn dispatch_script_command_with_runtime(runtime: &Runtime, command: ScriptCommand) {
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
        dev_console_state.write_line("script command registry service is missing".to_owned());
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
