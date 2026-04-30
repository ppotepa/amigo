use super::super::super::*;
use super::super::{AppScriptCommandContext, ScriptCommandHandler};
use std::path::{Component, Path, PathBuf};

pub(super) struct DebugScriptCommandHandler;

impl ScriptCommandHandler for DebugScriptCommandHandler {
    fn name(&self) -> &'static str {
        "debug"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(command.namespace.as_str(), "debug")
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'_>, command: ScriptCommand) {
        match (command.name.as_str(), command.arguments.as_slice()) {
            ("log", [line]) => {
                ctx.dev_console_state.write_line(format!("script: {line}"));
            }
            ("warn", [line]) => {
                ctx.dev_console_state
                    .write_line(format!("script warning: {line}"));
            }
            ("write-text", [relative_path, contents])
            | ("write_text", [relative_path, contents]) => match dev_export_path(relative_path) {
                Some(path) => match write_text_file(&path, contents) {
                    Ok(()) => ctx
                        .dev_console_state
                        .write_line(format!("script wrote text export `{}`", path.display())),
                    Err(error) => ctx.dev_console_state.write_line(format!(
                        "failed to write text export `{}`: {error}",
                        path.display()
                    )),
                },
                None => ctx
                    .dev_console_state
                    .write_line(format!("refused unsafe text export path `{relative_path}`")),
            },
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
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
