use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
};

use amigo_core::{AmigoError, AmigoResult};
use amigo_modding::ModCatalog;
use amigo_scene::{
    SceneCommand, SceneEvent, SceneEventQueue, SceneService, ScriptComponentParamValueSceneCommand,
    format_scene_command,
};
use amigo_scripting_api::{
    ScriptComponentDefinition, ScriptComponentService, ScriptParams, ScriptRuntimeService,
    ScriptSourceContext, ScriptValue,
};

pub struct RhaiSceneCommandHandler;

pub struct RhaiSceneCommandContext<'a> {
    pub mod_catalog: &'a ModCatalog,
    pub script_runtime: &'a ScriptRuntimeService,
    pub scene_service: &'a SceneService,
    pub scene_event_queue: &'a SceneEventQueue,
    pub script_component_service: &'a ScriptComponentService,
}

pub struct RhaiSceneCommandOutcome {
    pub entity_name: String,
    pub source_mod: String,
}

pub fn can_handle_rhai_scene_command(command: &SceneCommand) -> bool {
    matches!(command, SceneCommand::QueueScriptComponent { .. })
}

pub fn handle_rhai_scene_command(
    ctx: RhaiSceneCommandContext<'_>,
    command: SceneCommand,
) -> AmigoResult<RhaiSceneCommandOutcome> {
    match command {
        SceneCommand::QueueScriptComponent { command } => {
            let pending_source_name = script_component_source_name(
                &command.source_mod,
                &command.entity_name,
                &command.script,
            );
            let discovered_mod =
                ctx.mod_catalog
                    .mod_by_id(&command.source_mod)
                    .ok_or_else(|| {
                        script_component_lifecycle_error(
                            &command.entity_name,
                            &command.script,
                            &pending_source_name,
                            "load",
                            format!("references unloaded mod `{}`", command.source_mod),
                        )
                    })?;
            let script_path = discovered_mod.root_path.join(&command.script);
            let relative_script_path =
                relative_path_within_root(&discovered_mod.root_path, &script_path).map_err(
                    |error| {
                        script_component_lifecycle_error(
                            &command.entity_name,
                            &command.script,
                            &pending_source_name,
                            "load",
                            error,
                        )
                    },
                )?;
            validate_script_path(
                ctx.script_runtime,
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

            ctx.script_runtime
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
            ctx.script_runtime
                .validate_source(&source)
                .map_err(|error| {
                    script_component_lifecycle_error(
                        &command.entity_name,
                        &relative_script_path,
                        &source_name,
                        "validate",
                        error,
                    )
                })?;
            ctx.script_runtime
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
            ctx.script_runtime
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

            Ok(RhaiSceneCommandOutcome {
                entity_name: command.entity_name,
                source_mod: command.source_mod,
            })
        }
        other => Err(AmigoError::Message(format!(
            "rhai scene command handler cannot handle {}",
            format_scene_command(&other)
        ))),
    }
}

fn relative_path_within_root(root_path: &Path, absolute_path: &Path) -> AmigoResult<PathBuf> {
    let relative_path = absolute_path.strip_prefix(root_path).map_err(|_| {
        AmigoError::Message(format!(
            "script path `{}` must stay within mod root `{}`",
            absolute_path.display(),
            root_path.display()
        ))
    })?;

    if relative_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(AmigoError::Message(format!(
            "script path `{}` resolved to an invalid relative path `{}`",
            absolute_path.display(),
            relative_path.display()
        )));
    }

    Ok(relative_path.to_path_buf())
}

fn validate_script_path(
    script_runtime: &ScriptRuntimeService,
    script_path: &Path,
    owner_label: &str,
) -> AmigoResult<()> {
    let extension = script_path
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or_else(|| {
            AmigoError::Message(format!(
                "{owner_label} `{}` has no file extension",
                script_path.display()
            ))
        })?;

    if !script_runtime.supports_extension(extension) {
        return Err(AmigoError::Message(format!(
            "{owner_label} `{}` is not supported by `{}`",
            script_path.display(),
            script_runtime.backend_name()
        )));
    }

    Ok(())
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

impl amigo_scene::RuntimeSceneCommandHandler for RhaiSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        can_handle_rhai_scene_command(command)
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let mod_catalog = runtime.required::<ModCatalog>()?;
        let script_runtime = runtime.required::<ScriptRuntimeService>()?;
        let scene_service = runtime.required::<SceneService>()?;
        let scene_event_queue = runtime.required::<SceneEventQueue>()?;
        let script_component_service = runtime.required::<ScriptComponentService>()?;

        handle_rhai_scene_command(
            RhaiSceneCommandContext {
                mod_catalog: mod_catalog.as_ref(),
                script_runtime: script_runtime.as_ref(),
                scene_service: scene_service.as_ref(),
                scene_event_queue: scene_event_queue.as_ref(),
                script_component_service: script_component_service.as_ref(),
            },
            command,
        )?;
        Ok(())
    }
}
