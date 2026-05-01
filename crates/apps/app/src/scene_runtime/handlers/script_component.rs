use std::fs;

use amigo_scene::{SceneCommand, ScriptComponentParamValueSceneCommand};
use amigo_scripting_api::ScriptSourceContext;

use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::*;

pub(crate) struct SceneScriptComponentCommandHandler;

impl SceneCommandHandler for SceneScriptComponentCommandHandler {
    fn name(&self) -> &'static str {
        "scene-script-component"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueScriptComponent { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueScriptComponent { command } => {
                let pending_source_name = script_component_source_name(
                    &command.source_mod,
                    &command.entity_name,
                    &command.script,
                );
                let mod_catalog = required::<ModCatalog>(ctx.runtime)?;
                let script_runtime = required::<ScriptRuntimeService>(ctx.runtime)?;
                let discovered_mod =
                    mod_catalog.mod_by_id(&command.source_mod).ok_or_else(|| {
                        script_component_lifecycle_error(
                            &command.entity_name,
                            &command.script,
                            &pending_source_name,
                            "load",
                            format!("references unloaded mod `{}`", command.source_mod),
                        )
                    })?;
                let script_path = discovered_mod.root_path.join(&command.script);
                let relative_script_path = crate::app_helpers::relative_path_within_root(
                    &discovered_mod.root_path,
                    &script_path,
                )
                .map_err(|error| {
                    script_component_lifecycle_error(
                        &command.entity_name,
                        &command.script,
                        &pending_source_name,
                        "load",
                        error,
                    )
                })?;
                crate::app_helpers::validate_script_path(
                    script_runtime.as_ref(),
                    &relative_script_path,
                    &format!("script component `{}`", command.entity_name),
                )
                .map_err(|error| {
                    script_component_lifecycle_error(
                        &command.entity_name,
                        &relative_script_path,
                        &pending_source_name,
                        "validate",
                        error,
                    )
                })?;
                let source = fs::read_to_string(&script_path).map_err(|error| {
                    script_component_lifecycle_error(
                        &command.entity_name,
                        &relative_script_path,
                        &pending_source_name,
                        "load",
                        error,
                    )
                })?;
                let source_name = script_component_source_name(
                    &command.source_mod,
                    &command.entity_name,
                    &relative_script_path,
                );
                let context = ScriptSourceContext {
                    source_name: source_name.clone(),
                    mod_root_path: discovered_mod.root_path.clone(),
                    script_dir_path: script_path
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| discovered_mod.root_path.clone()),
                };
                let params = script_params_from_scene(command.params);

                script_runtime
                    .set_source_context(context)
                    .map_err(|error| {
                        script_component_lifecycle_error(
                            &command.entity_name,
                            &relative_script_path,
                            &source_name,
                            "load",
                            error,
                        )
                    })?;
                script_runtime.validate_source(&source).map_err(|error| {
                    script_component_lifecycle_error(
                        &command.entity_name,
                        &relative_script_path,
                        &source_name,
                        "validate",
                        error,
                    )
                })?;
                script_runtime
                    .execute_source(&source_name, &source)
                    .map_err(|error| {
                        script_component_lifecycle_error(
                            &command.entity_name,
                            &relative_script_path,
                            &source_name,
                            "execute",
                            error,
                        )
                    })?;
                script_runtime
                    .call_component_on_attach(&source_name, &command.entity_name, &params)
                    .map_err(|error| {
                        script_component_lifecycle_error(
                            &command.entity_name,
                            &relative_script_path,
                            &source_name,
                            "on_attach",
                            error,
                        )
                    })?;

                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                ctx.script_component_service
                    .queue(ScriptComponentDefinition {
                        source_mod: command.source_mod.clone(),
                        entity_name: command.entity_name.clone(),
                        source_name: source_name.clone(),
                        script: relative_script_path,
                        params,
                    });
                ctx.scene_event_queue
                    .publish(SceneEvent::ScriptComponentQueued {
                        entity_id: entity.raw(),
                        entity_name: command.entity_name.clone(),
                        source_name,
                    });
                ctx.dev_console_state.write_line(format!(
                    "queued script component `{}` from mod `{}`",
                    command.entity_name, command.source_mod
                ));
                Ok(())
            }
            other => Err(AmigoError::Message(format!(
                "{} cannot handle {}",
                self.name(),
                amigo_scene::format_scene_command(&other)
            ))),
        }
    }
}

fn script_component_source_name(source_mod: &str, entity_name: &str, script: &Path) -> String {
    format!(
        "component:{}:{}:{}",
        source_mod,
        entity_name,
        script.display()
    )
}

fn script_component_lifecycle_error(
    entity_name: &str,
    script: &Path,
    source_name: &str,
    phase: &str,
    error: impl std::fmt::Display,
) -> AmigoError {
    AmigoError::Message(format!(
        "script component lifecycle phase `{phase}` failed for entity `{entity_name}` (script path `{}`, source name `{source_name}`): {error}",
        script.display()
    ))
}

fn script_params_from_scene(
    params: BTreeMap<String, ScriptComponentParamValueSceneCommand>,
) -> ScriptParams {
    params
        .into_iter()
        .map(|(key, value)| {
            let value = match value {
                ScriptComponentParamValueSceneCommand::Bool(value) => ScriptValue::Bool(value),
                ScriptComponentParamValueSceneCommand::Int(value) => ScriptValue::Int(value),
                ScriptComponentParamValueSceneCommand::Float(value) => ScriptValue::Float(value),
                ScriptComponentParamValueSceneCommand::String(value) => ScriptValue::String(value),
            };
            (key, value)
        })
        .collect()
}
