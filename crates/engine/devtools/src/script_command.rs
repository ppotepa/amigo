use std::path::{Component, Path, PathBuf};

use amigo_core::{AmigoResult, RuntimeDiagnostics};
use amigo_runtime::Runtime;
use amigo_scripting_api::{
    DevConsoleState, RuntimeScriptCommandHandler, ScriptCommand, ScriptEvent, ScriptEventQueue,
};

pub struct DebugScriptCommandHandler;

impl RuntimeScriptCommandHandler for DebugScriptCommandHandler {
    fn name(&self) -> &'static str {
        "debug"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        command.namespace == "debug"
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let dev_console_state = runtime.required::<DevConsoleState>()?;

        match (command.name.as_str(), command.arguments.as_slice()) {
            ("log", [line]) => {
                dev_console_state.write_line(format!("script: {line}"));
            }
            ("warn", [line]) => {
                dev_console_state.write_line(format!("script warning: {line}"));
            }
            ("write-text", [relative_path, contents])
            | ("write_text", [relative_path, contents]) => match dev_export_path(relative_path) {
                Some(path) => match write_text_file(&path, contents) {
                    Ok(()) => dev_console_state
                        .write_line(format!("script wrote text export `{}`", path.display())),
                    Err(error) => dev_console_state.write_line(format!(
                        "failed to write text export `{}`: {error}",
                        path.display()
                    )),
                },
                None => dev_console_state
                    .write_line(format!("refused unsafe text export path `{relative_path}`")),
            },
            _ => dev_console_state.write_line(format!(
                "{} could not handle command: {}:{} {:?}",
                self.name(),
                command.namespace,
                command.name,
                command.arguments,
            )),
        }

        Ok(())
    }
}

pub struct DevShellScriptCommandHandler;

impl RuntimeScriptCommandHandler for DevShellScriptCommandHandler {
    fn name(&self) -> &'static str {
        "dev-shell"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        command.namespace == "dev-shell"
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let dev_console_state = runtime.required::<DevConsoleState>()?;
        let script_event_queue = runtime.required::<ScriptEventQueue>()?;
        let diagnostics = runtime.required::<RuntimeDiagnostics>()?;

        match (command.name.as_str(), command.arguments.as_slice()) {
            ("refresh-diagnostics", [target_mod]) => {
                dev_console_state.write_line(format!(
                    "diagnostics refreshed for mod={} window={} input={} render={} script={}",
                    target_mod,
                    diagnostics.window_backend,
                    diagnostics.input_backend,
                    diagnostics.render_backend,
                    diagnostics.script_backend
                ));
                script_event_queue.publish(ScriptEvent::new(
                    "dev-shell.diagnostics-refreshed",
                    vec![target_mod.clone()],
                ));
            }
            _ => dev_console_state.write_line(format!(
                "{} could not handle command: {}:{} {:?}",
                self.name(),
                command.namespace,
                command.name,
                command.arguments,
            )),
        }

        Ok(())
    }
}

fn dev_export_path(relative_path: &str) -> Option<PathBuf> {
    let relative = Path::new(relative_path);
    if relative.is_absolute() {
        return None;
    }
    if !relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        return None;
    }
    Some(
        PathBuf::from("target")
            .join("amigo-dev-exports")
            .join(relative),
    )
}

fn write_text_file(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, contents)
}
