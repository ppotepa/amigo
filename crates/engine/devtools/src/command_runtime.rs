use std::sync::Arc;

use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;
use amigo_scripting_api::{
    DevConsoleCommand, DevConsoleEvalResult, DevConsoleOutputLevel, DevConsoleScriptContext,
    DevConsoleState, ScriptRuntimeService,
};

use crate::{
    classify_console_input, parse_console_command, should_try_rhai_fallback,
    ConsoleCommandDescriptor, ConsoleCommandRegistry, ConsoleCommandResult, ConsoleCommandSpec,
    ConsoleInputKind, ParsedConsoleCommand,
};

pub trait RuntimeConsoleCommandHandler: Send + Sync {
    fn name(&self) -> &'static str;

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor>;

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool;

    fn handle(
        &self,
        ctx: &DevConsoleCommandContext<'_>,
        command: ParsedConsoleCommand,
    ) -> ConsoleCommandResult;
}

impl<T> ConsoleCommandSpec for T
where
    T: RuntimeConsoleCommandHandler + ?Sized,
{
    fn name(&self) -> &'static str {
        RuntimeConsoleCommandHandler::name(self)
    }

    fn descriptors(&self) -> Vec<ConsoleCommandDescriptor> {
        RuntimeConsoleCommandHandler::descriptors(self)
    }

    fn can_handle(&self, command: &ParsedConsoleCommand) -> bool {
        RuntimeConsoleCommandHandler::can_handle(self, command)
    }
}

pub type RuntimeConsoleCommandRegistry = ConsoleCommandRegistry<dyn RuntimeConsoleCommandHandler>;

pub fn register_runtime_console_command_handler<H>(
    registry: &RuntimeConsoleCommandRegistry,
    handler: H,
) where
    H: RuntimeConsoleCommandHandler + 'static,
{
    registry.register_arc(Arc::new(handler));
}

pub struct DevConsoleCommandContext<'a> {
    pub runtime: &'a Runtime,
    pub console: &'a DevConsoleState,
    pub registry: &'a RuntimeConsoleCommandRegistry,
}

impl<'a> DevConsoleCommandContext<'a> {
    pub fn required<T: Send + Sync + 'static>(&self) -> AmigoResult<Arc<T>> {
        self.runtime.required::<T>()
    }
}

pub fn dispatch_console_command(runtime: &Runtime, command: DevConsoleCommand) {
    let Ok(console) = runtime.required::<DevConsoleState>() else {
        return;
    };
    let Ok(registry) = runtime.required::<RuntimeConsoleCommandRegistry>() else {
        console.write_line("error: console command registry is not registered");
        return;
    };

    console.record_command(command.line.clone());

    match classify_console_input(&command.line) {
        ConsoleInputKind::Empty => return,
        ConsoleInputKind::PreferRhai => {
            console.write_console_log("route=prefer_rhai fallback=eval_console");
            write_console_result(console.as_ref(), eval_console_fallback(runtime, &command.line));
            return;
        }
        ConsoleInputKind::PreferCommand => {}
    }

    let Some(parsed) = parse_console_command(&command.line) else {
        return;
    };
    console.write_console_log(format!(
        "route=prefer_command parsed_name={:?} args={:?}",
        parsed.name, parsed.args
    ));

    let ctx = DevConsoleCommandContext {
        runtime,
        console: console.as_ref(),
        registry: registry.as_ref(),
    };
    let result = registry
        .handler_for(&parsed)
        .map(|handler| handler.handle(&ctx, parsed.clone()))
        .unwrap_or_else(|| ConsoleCommandResult::unknown(parsed.raw.clone()));

    let result = match result {
        ConsoleCommandResult::Unknown(raw) if should_try_rhai_fallback(&raw) => {
            console.write_console_log(format!(
                "route=unknown_command fallback=eval_console raw={raw:?}"
            ));
            eval_console_fallback(runtime, &raw)
        }
        other => other,
    };

    write_console_result(console.as_ref(), result);
}

fn eval_console_fallback(runtime: &Runtime, source: &str) -> ConsoleCommandResult {
    let Some(script_runtime) = runtime.resolve::<ScriptRuntimeService>() else {
        return ConsoleCommandResult::unknown(source.to_owned());
    };

    let scene_id = runtime
        .resolve::<SceneService>()
        .and_then(|scene| scene.selected_scene())
        .map(|scene| scene.as_str().to_owned());

    let context = DevConsoleScriptContext::new(scene_id);

    match script_runtime.eval_console(context, source) {
        Ok(DevConsoleEvalResult::Unit) => ConsoleCommandResult::Silent,
        Ok(DevConsoleEvalResult::Value(value)) => ConsoleCommandResult::ok(value),
        Err(error) => ConsoleCommandResult::error(error.to_string()),
    }
}

fn write_console_result(console: &DevConsoleState, result: ConsoleCommandResult) {
    match result {
        ConsoleCommandResult::Ok(message) => {
            console.write_line_with_level(message, DevConsoleOutputLevel::Success)
        }
        ConsoleCommandResult::Error(message) => {
            console.write_line_with_level(format!("error: {message}"), DevConsoleOutputLevel::Error)
        }
        ConsoleCommandResult::Unknown(raw) => console.write_line_with_level(
            format!("unknown command: {raw}"),
            DevConsoleOutputLevel::Warning,
        ),
        ConsoleCommandResult::Silent => {}
    }
}

