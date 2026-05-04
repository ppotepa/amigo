//! App-side scene command runtime.
//! It loads scene documents, builds hydration plans, and dispatches scene commands into domain services.

use super::*;
use amigo_scene::ActivationSetSceneService;

/// Shared context object passed into scene command handlers.
mod context;
/// Registry and dispatch plumbing for scene command handlers.
mod dispatcher;
/// Domain-specific handlers for hydrated scene commands.
mod handlers;
/// Helpers that synchronize runtime UI support data with loaded scenes.
mod ui_support;

use context::AppSceneCommandContext;
use dispatcher::SceneCommandHandlerRegistry;
pub(crate) use dispatcher::SceneCommandRuntimePlugin;

pub(crate) fn current_loaded_scene_document_summary(
    runtime: &Runtime,
) -> AmigoResult<Option<LoadedSceneDocumentSummary>> {
    let hydrated_scene_state = required::<HydratedSceneState>(runtime)?;
    let scene_transition_service = required::<SceneTransitionService>(runtime)?;
    let snapshot = hydrated_scene_state.snapshot();
    let transition_snapshot = scene_transition_service.snapshot();
    let (Some(source_mod), Some(scene_id), Some(relative_path)) = (
        snapshot.source_mod,
        snapshot.scene_id,
        snapshot.relative_document_path,
    ) else {
        return Ok(None);
    };

    Ok(Some(LoadedSceneDocumentSummary {
        source_mod,
        scene_id,
        relative_path,
        entity_names: snapshot.entity_names,
        component_kinds: snapshot.component_kinds,
        transition_ids: transition_snapshot.transition_ids,
    }))
}

pub(super) fn load_scene_document_for_mod(
    runtime: &Runtime,
    root_mod: &str,
    scene_id: &str,
) -> AmigoResult<Option<LoadedSceneDocument>> {
    let mod_catalog = required::<ModCatalog>(runtime)?;
    let discovered_mod = mod_catalog.mod_by_id(root_mod).ok_or_else(|| {
        AmigoError::Message(format!(
            "root mod `{root_mod}` was not loaded by the current bootstrap selection"
        ))
    })?;
    let scene_manifest = discovered_mod.scene_by_id(scene_id).ok_or_else(|| {
        AmigoError::Message(format!(
            "scene `{scene_id}` was not declared by root mod `{root_mod}`"
        ))
    })?;
    let document_path = discovered_mod
        .scene_document_path(scene_id)
        .ok_or_else(|| {
            AmigoError::Message(format!(
                "scene `{scene_id}` for mod `{root_mod}` has no resolved document path"
            ))
        })?;
    if !document_path.is_file() {
        return if scene_manifest.document.is_some() {
            Err(AmigoError::Message(format!(
                "scene `{scene_id}` for mod `{root_mod}` declares document `{}` but the file does not exist",
                document_path.display()
            )))
        } else {
            Err(AmigoError::Message(format!(
                "scene `{scene_id}` for mod `{root_mod}` is missing default document `{}`",
                document_path.display()
            )))
        };
    }
    let relative_document_path =
        crate::app_helpers::relative_path_within_root(&discovered_mod.root_path, &document_path)?;
    let document = amigo_scene::load_scene_document_from_path(&document_path)
        .map_err(|error| AmigoError::Message(error.to_string()))?;

    if document.scene.id != scene_id {
        return Err(AmigoError::Message(format!(
            "scene document `{}` declares id `{}` but manifest selected `{scene_id}`",
            document_path.display(),
            document.scene.id
        )));
    }

    let hydration_plan = amigo_scene::build_scene_hydration_plan(root_mod, &document)
        .map_err(|error| AmigoError::Message(error.to_string()))?;
    let transition_plan = amigo_scene::build_scene_transition_plan(root_mod, &document)
        .map_err(|error| AmigoError::Message(error.to_string()))?;

    let component_kinds = document
        .component_kind_counts()
        .into_iter()
        .map(|(kind, count)| format!("{kind} x{count}"))
        .collect::<Vec<_>>();
    let transition_ids = transition_plan
        .as_ref()
        .map(|plan| {
            plan.transitions
                .iter()
                .map(|transition| transition.id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(Some(LoadedSceneDocument {
        summary: LoadedSceneDocumentSummary {
            source_mod: root_mod.to_owned(),
            scene_id: scene_id.to_owned(),
            relative_path: relative_document_path,
            entity_names: document.entity_names(),
            component_kinds,
            transition_ids,
        },
        hydration_plan,
        transition_plan,
    }))
}

pub(super) fn queue_scene_document_hydration(
    scene_command_queue: &SceneCommandQueue,
    dev_console_state: &DevConsoleState,
    hydrated_scene_state: &HydratedSceneState,
    scene_transition_service: &SceneTransitionService,
    loaded_scene_document: &LoadedSceneDocument,
) {
    hydrated_scene_state.replace(amigo_scene::HydratedSceneSnapshot {
        source_mod: Some(loaded_scene_document.summary.source_mod.clone()),
        scene_id: Some(loaded_scene_document.summary.scene_id.clone()),
        relative_document_path: Some(loaded_scene_document.summary.relative_path.clone()),
        entity_names: loaded_scene_document.summary.entity_names.clone(),
        component_kinds: loaded_scene_document.summary.component_kinds.clone(),
    });
    scene_transition_service.activate(loaded_scene_document.transition_plan.clone());

    for command in &loaded_scene_document.hydration_plan.commands {
        scene_command_queue.submit(command.clone());
    }

    dev_console_state.write_line(format!(
        "queued scene document hydration for `{}` with {} commands",
        loaded_scene_document.summary.scene_id,
        loaded_scene_document.hydration_plan.commands.len()
    ));
}

pub(crate) fn apply_scene_command(runtime: &Runtime, command: SceneCommand) -> AmigoResult<()> {
    let scene_command_queue = required::<SceneCommandQueue>(runtime)?;
    let launch_selection = required::<LaunchSelection>(runtime)?;
    let hydrated_scene_state = required::<HydratedSceneState>(runtime)?;
    let scene_transition_service = required::<SceneTransitionService>(runtime)?;
    let scene_service = required::<SceneService>(runtime)?;
    let entity_pool_scene_service = required::<EntityPoolSceneService>(runtime)?;
    let lifetime_scene_service = required::<LifetimeSceneService>(runtime)?;
    let scene_event_queue = required::<SceneEventQueue>(runtime)?;
    let dev_console_state = required::<DevConsoleState>(runtime)?;
    let asset_catalog = required::<AssetCatalog>(runtime)?;
    let sprite_scene_service = required::<SpriteSceneService>(runtime)?;
    let text_scene_service = required::<Text2dSceneService>(runtime)?;
    let vector_scene_service = required::<VectorSceneService>(runtime)?;
    let physics_scene_service = required::<Physics2dSceneService>(runtime)?;
    let tilemap_scene_service = required::<TileMap2dSceneService>(runtime)?;
    let motion_scene_service = required::<Motion2dSceneService>(runtime)?;
    let input_action_service = required::<InputActionService>(runtime)?;
    let behavior_scene_service = required::<BehaviorSceneService>(runtime)?;
    let event_pipeline_service = required::<EventPipelineService>(runtime)?;
    let script_component_service = required::<ScriptComponentService>(runtime)?;
    let particle2d_scene_service = required::<Particle2dSceneService>(runtime)?;
    let camera_follow_scene_service = required::<CameraFollow2dSceneService>(runtime)?;
    let parallax_scene_service = required::<Parallax2dSceneService>(runtime)?;
    let mesh_scene_service = required::<MeshSceneService>(runtime)?;
    let text3d_scene_service = required::<Text3dSceneService>(runtime)?;
    let material_scene_service = required::<MaterialSceneService>(runtime)?;
    let ui_scene_service = required::<UiSceneService>(runtime)?;
    let ui_state_service = required::<UiStateService>(runtime)?;
    let ui_model_binding_service = required::<UiModelBindingService>(runtime)?;
    let ui_theme_service = required::<UiThemeService>(runtime)?;
    let audio_scene_service = required::<AudioSceneService>(runtime)?;
    let activation_set_scene_service = required::<ActivationSetSceneService>(runtime)?;

    let ctx = AppSceneCommandContext {
        runtime,
        scene_command_queue: scene_command_queue.as_ref(),
        launch_selection: launch_selection.as_ref(),
        hydrated_scene_state: hydrated_scene_state.as_ref(),
        scene_transition_service: scene_transition_service.as_ref(),
        scene_service: scene_service.as_ref(),
        entity_pool_scene_service: entity_pool_scene_service.as_ref(),
        lifetime_scene_service: lifetime_scene_service.as_ref(),
        scene_event_queue: scene_event_queue.as_ref(),
        dev_console_state: dev_console_state.as_ref(),
        asset_catalog: asset_catalog.as_ref(),
        sprite_scene_service: sprite_scene_service.as_ref(),
        text_scene_service: text_scene_service.as_ref(),
        vector_scene_service: vector_scene_service.as_ref(),
        physics_scene_service: physics_scene_service.as_ref(),
        tilemap_scene_service: tilemap_scene_service.as_ref(),
        motion_scene_service: motion_scene_service.as_ref(),
        input_action_service: input_action_service.as_ref(),
        behavior_scene_service: behavior_scene_service.as_ref(),
        event_pipeline_service: event_pipeline_service.as_ref(),
        script_component_service: script_component_service.as_ref(),
        particle2d_scene_service: particle2d_scene_service.as_ref(),
        camera_follow_scene_service: camera_follow_scene_service.as_ref(),
        parallax_scene_service: parallax_scene_service.as_ref(),
        mesh_scene_service: mesh_scene_service.as_ref(),
        text3d_scene_service: text3d_scene_service.as_ref(),
        material_scene_service: material_scene_service.as_ref(),
        ui_scene_service: ui_scene_service.as_ref(),
        ui_state_service: ui_state_service.as_ref(),
        ui_model_binding_service: ui_model_binding_service.as_ref(),
        ui_theme_service: ui_theme_service.as_ref(),
        audio_scene_service: audio_scene_service.as_ref(),
        activation_set_scene_service: activation_set_scene_service.as_ref(),
    };

    let registry = required::<SceneCommandHandlerRegistry>(runtime)?;
    amigo_runtime::HandlerDispatcher::new(registry)
        .dispatch_first(|handler| {
            handler
                .can_handle(&command)
                .then(|| handler.handle(&ctx, command.clone()))
        })
        .unwrap_or_else(|| {
            Err(AmigoError::Message(format!(
                "unhandled scene command in dispatcher: {}",
                amigo_scene::format_scene_command(&command)
            )))
        })
}

pub(super) fn clear_runtime_scene_content(
    hydrated_scene_state: &HydratedSceneState,
    scene_service: &SceneService,
    dev_console_state: &DevConsoleState,
    sprite_scene_service: &SpriteSceneService,
    text_scene_service: &Text2dSceneService,
    vector_scene_service: &VectorSceneService,
    physics_scene_service: &Physics2dSceneService,
    tilemap_scene_service: &TileMap2dSceneService,
    motion_scene_service: &Motion2dSceneService,
    particle2d_scene_service: &Particle2dSceneService,
    input_action_service: &InputActionService,
    behavior_scene_service: &BehaviorSceneService,
    event_pipeline_service: &EventPipelineService,
    script_component_service: &ScriptComponentService,
    script_trace_service: &ScriptTraceService,
    entity_pool_scene_service: &EntityPoolSceneService,
    lifetime_scene_service: &LifetimeSceneService,
    camera_follow_scene_service: &CameraFollow2dSceneService,
    parallax_scene_service: &Parallax2dSceneService,
    mesh_scene_service: &MeshSceneService,
    text3d_scene_service: &Text3dSceneService,
    material_scene_service: &MaterialSceneService,
    ui_scene_service: &UiSceneService,
    ui_state_service: &UiStateService,
    ui_model_binding_service: &UiModelBindingService,
    ui_theme_service: &UiThemeService,
    audio_scene_service: &AudioSceneService,
    audio_state_service: &AudioStateService,
    audio_mixer_service: &AudioMixerService,
    audio_output_service: &AudioOutputBackendService,
    activation_set_scene_service: &ActivationSetSceneService,
    state_service: &amigo_state::SceneStateService,
    timer_service: &amigo_state::SceneTimerService,
) {
    let previous = hydrated_scene_state.clear();

    if !previous.entity_names.is_empty() {
        let removed = scene_service.remove_entities_by_name(&previous.entity_names);
        dev_console_state.write_line(format!(
            "removed {removed} hydrated scene entities from `{}`",
            previous.scene_id.as_deref().unwrap_or("unknown")
        ));
    }

    sprite_scene_service.clear();
    text_scene_service.clear();
    vector_scene_service.clear();
    physics_scene_service.clear();
    tilemap_scene_service.clear();
    motion_scene_service.clear();
    particle2d_scene_service.clear();
    input_action_service.clear();
    behavior_scene_service.clear();
    event_pipeline_service.clear();
    script_component_service.clear();
    script_trace_service.clear();
    entity_pool_scene_service.clear();
    lifetime_scene_service.clear();
    camera_follow_scene_service.clear();
    parallax_scene_service.clear();
    mesh_scene_service.clear();
    text3d_scene_service.clear();
    material_scene_service.clear();
    ui_scene_service.clear();
    ui_state_service.clear();
    ui_model_binding_service.clear();
    ui_theme_service.clear();
    audio_scene_service.clear();
    audio_state_service.clear();
    audio_mixer_service.clear();
    audio_output_service.clear_buffer();
    activation_set_scene_service.clear();
    state_service.clear_scene();
    timer_service.reset_scene();
}

pub(super) fn clear_runtime_scene_content_with_runtime(runtime: &Runtime) -> AmigoResult<()> {
    let script_runtime = required::<ScriptRuntimeService>(runtime)?;
    let script_component_service = required::<ScriptComponentService>(runtime)?;
    for component in script_component_service.components() {
        script_runtime
            .call_component_on_detach(
                &component.source_name,
                &component.entity_name,
                &component.params,
            )
            .map_err(|error| {
                script_component_lifecycle_error(
                    &component.entity_name,
                    &component.script,
                    &component.source_name,
                    "on_detach",
                    error,
                )
            })?;
        script_runtime
            .unload_source(&component.source_name)
            .map_err(|error| {
                script_component_lifecycle_error(
                    &component.entity_name,
                    &component.script,
                    &component.source_name,
                    "unload",
                    error,
                )
            })?;
    }

    clear_runtime_scene_content(
        required::<HydratedSceneState>(runtime)?.as_ref(),
        required::<SceneService>(runtime)?.as_ref(),
        required::<DevConsoleState>(runtime)?.as_ref(),
        required::<SpriteSceneService>(runtime)?.as_ref(),
        required::<Text2dSceneService>(runtime)?.as_ref(),
        required::<VectorSceneService>(runtime)?.as_ref(),
        required::<Physics2dSceneService>(runtime)?.as_ref(),
        required::<TileMap2dSceneService>(runtime)?.as_ref(),
        required::<Motion2dSceneService>(runtime)?.as_ref(),
        required::<Particle2dSceneService>(runtime)?.as_ref(),
        required::<InputActionService>(runtime)?.as_ref(),
        required::<BehaviorSceneService>(runtime)?.as_ref(),
        required::<EventPipelineService>(runtime)?.as_ref(),
        required::<ScriptComponentService>(runtime)?.as_ref(),
        required::<ScriptTraceService>(runtime)?.as_ref(),
        required::<EntityPoolSceneService>(runtime)?.as_ref(),
        required::<LifetimeSceneService>(runtime)?.as_ref(),
        required::<CameraFollow2dSceneService>(runtime)?.as_ref(),
        required::<Parallax2dSceneService>(runtime)?.as_ref(),
        required::<MeshSceneService>(runtime)?.as_ref(),
        required::<Text3dSceneService>(runtime)?.as_ref(),
        required::<MaterialSceneService>(runtime)?.as_ref(),
        required::<UiSceneService>(runtime)?.as_ref(),
        required::<UiStateService>(runtime)?.as_ref(),
        required::<UiModelBindingService>(runtime)?.as_ref(),
        required::<UiThemeService>(runtime)?.as_ref(),
        required::<AudioSceneService>(runtime)?.as_ref(),
        required::<AudioStateService>(runtime)?.as_ref(),
        required::<AudioMixerService>(runtime)?.as_ref(),
        required::<AudioOutputBackendService>(runtime)?.as_ref(),
        required::<ActivationSetSceneService>(runtime)?.as_ref(),
        required::<amigo_state::SceneStateService>(runtime)?.as_ref(),
        required::<amigo_state::SceneTimerService>(runtime)?.as_ref(),
    );
    Ok(())
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
