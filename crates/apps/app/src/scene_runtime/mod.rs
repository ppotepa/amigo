//! App-side scene command runtime.
//! It loads scene documents, builds hydration plans, and dispatches scene commands into domain services.

use super::*;
use amigo_runtime::EngineSchedulerMode;
use amigo_scene::ActivationSetSceneService;
use amigo_scene::CompiledSceneDocument;
use amigo_scene::SceneSchedulingDocument;
use amigo_session::{
    RuntimeSession, SceneLoadRequest, SceneSessionLoadedDocument, SceneSessionService,
};

/// Registry and dispatch plumbing for scene command handlers.
mod dispatcher;
/// Helpers that synchronize runtime UI support data with loaded scenes.
mod ui_support;

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
    let compiled = amigo_scene::compile_scene_document_from_path(
        &document_path,
        &discovered_mod.root_path,
        root_mod,
    )
    .map_err(|error| AmigoError::Message(error.to_string()))?;
    apply_compiled_scene_scheduling(runtime, &discovered_mod.root_path, scene_id, &compiled)?;
    let document = compiled.document;

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

// Internal migration seam: app-hosted scene loading remains in this module while
// P0.1 exposes it through `RuntimeSession` lifecycle tracking.
pub(crate) fn load_scene_document_for_session(
    session: &mut RuntimeSession,
    root_mod: &str,
    scene_id: &str,
) -> AmigoResult<Option<LoadedSceneDocument>> {
    let request = SceneLoadRequest::new(root_mod, scene_id);
    session.begin_scene_load(&request);

    match load_scene_document_for_mod(session.runtime(), root_mod, scene_id) {
        Ok(Some(loaded_scene_document)) => {
            session.complete_scene_load(scene_session_loaded_document_from_loaded(
                &loaded_scene_document,
            ));
            Ok(Some(loaded_scene_document))
        }
        Ok(None) => {
            session.fail_scene_load(
                &request,
                format!("scene `{scene_id}` for mod `{root_mod}` did not resolve to a document"),
            );
            Ok(None)
        }
        Err(error) => {
            session.fail_scene_load(&request, error.to_string());
            Err(error)
        }
    }
}

fn apply_compiled_scene_scheduling(
    runtime: &Runtime,
    mod_root_path: &Path,
    _scene_id: &str,
    compiled: &CompiledSceneDocument,
) -> AmigoResult<()> {
    let scheduling_service = required::<amigo_session::RuntimeSchedulingService>(runtime)?;
    let mut resolved = crate::scheduling::ResolvedSchedulingConfig::default();

    if let Some(mod_scheduling) = load_mod_level_scheduling(mod_root_path)? {
        apply_scene_scheduling_into_resolved(&mut resolved, mod_scheduling);
    }
    if let Some(scene_scheduling) = compiled.scheduling.clone() {
        apply_scene_scheduling_into_resolved(&mut resolved, scene_scheduling);
    }

    scheduling_service.set_config(resolved.clone());
    scheduling_service.set_mode(resolved.mode);
    Ok(())
}

fn load_mod_level_scheduling(mod_root_path: &Path) -> AmigoResult<Option<SceneSchedulingDocument>> {
    let yml_path = mod_root_path.join("scheduling.yml");
    let yaml_path = mod_root_path.join("scheduling.yaml");
    let path = if yml_path.is_file() {
        yml_path
    } else if yaml_path.is_file() {
        yaml_path
    } else {
        return Ok(None);
    };

    let raw = std::fs::read_to_string(&path).map_err(|error| {
        AmigoError::Message(format!("failed to read `{}`: {error}", path.display()))
    })?;
    let value = serde_yaml::from_str::<serde_yaml::Value>(&raw).map_err(|error| {
        AmigoError::Message(format!("failed to parse `{}`: {error}", path.display()))
    })?;

    let scheduling_value = value
        .as_mapping()
        .and_then(|mapping| mapping.get(serde_yaml::Value::String("scheduling".to_owned())))
        .cloned()
        .unwrap_or(value);

    let scheduling =
        serde_yaml::from_value::<SceneSchedulingDocument>(scheduling_value).map_err(|error| {
            AmigoError::Message(format!(
                "invalid scheduling document `{}`: {error}",
                path.display()
            ))
        })?;
    Ok(Some(scheduling))
}

fn apply_scene_scheduling_into_resolved(
    resolved: &mut crate::scheduling::ResolvedSchedulingConfig,
    scheduling: SceneSchedulingDocument,
) {
    if let Some(mode) = scheduling.mode.as_deref().and_then(parse_scheduler_mode) {
        resolved.mode = mode;
    }
    if let Some(max_workers) = scheduling.max_workers {
        resolved.max_workers = max_workers;
    }
    if let Some(allow_frame_latency) = scheduling.allow_frame_latency {
        resolved.allow_frame_latency = allow_frame_latency;
    }
    if scheduling.strict {
        resolved.deterministic = true;
    }

    for override_document in scheduling.overrides {
        resolved
            .overrides
            .push(crate::scheduling::ResolvedSchedulingOverride {
                target: override_document.target,
                lane: override_document.lane,
                priority: override_document.priority,
                parallelism: override_document.parallelism,
                allow_frame_latency: override_document.allow_frame_latency,
                quality_scale: override_document.quality_scale,
                budget_ms: override_document.budget_ms,
            });
    }
}

fn parse_scheduler_mode(value: &str) -> Option<EngineSchedulerMode> {
    match value {
        "single_thread" => Some(EngineSchedulerMode::SingleThread),
        "auto" => Some(EngineSchedulerMode::Auto),
        "hybrid" => Some(EngineSchedulerMode::Hybrid),
        "manual" => Some(EngineSchedulerMode::Manual),
        _ => None,
    }
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

fn register_scene_command_asset_references(
    asset_catalog: &amigo_assets::AssetCatalog,
    command: &SceneCommand,
) {
    match command {
        SceneCommand::QueueSprite2d { command } => {
            crate::app_helpers::register_mod_asset_reference(
                asset_catalog,
                &command.source_mod,
                &command.texture,
                "spritesheets",
                "sprite-sheet-2d",
            );
        }
        SceneCommand::QueueLayeredImage2d { command } => {
            crate::app_helpers::register_mod_asset_reference(
                asset_catalog,
                &command.source_mod,
                &command.asset,
                "layered-images",
                "layered-image-2d",
            );
        }
        SceneCommand::QueueTileMap2d { command } => {
            crate::app_helpers::register_mod_asset_reference(
                asset_catalog,
                &command.source_mod,
                &command.tileset,
                "tilemaps",
                "tilemap-2d",
            );
            if let Some(ruleset) = &command.ruleset {
                crate::app_helpers::register_mod_asset_reference(
                    asset_catalog,
                    &command.source_mod,
                    ruleset,
                    "tilemaps",
                    "tile-ruleset-2d",
                );
            }
            if let Some(sprite_sheet) = command
                .tileset
                .as_str()
                .split_once("/tilesets/")
                .map(|(sprite_sheet, _)| amigo_assets::AssetKey::new(sprite_sheet.to_owned()))
            {
                crate::app_helpers::register_mod_asset_reference(
                    asset_catalog,
                    &command.source_mod,
                    &sprite_sheet,
                    "spritesheets",
                    "sprite-sheet-2d",
                );
            }
        }
        SceneCommand::QueueText2d { command } => {
            crate::app_helpers::register_mod_asset_reference(
                asset_catalog,
                &command.source_mod,
                &command.font,
                "fonts",
                "font-2d",
            );
        }
        SceneCommand::QueueMesh3d { command } => {
            crate::app_helpers::register_mod_asset_reference(
                asset_catalog,
                &command.source_mod,
                &command.mesh_asset,
                "meshes",
                "mesh-3d",
            );
        }
        SceneCommand::QueueMaterial3d { command } => {
            if let Some(source) = &command.source {
                crate::app_helpers::register_mod_asset_reference(
                    asset_catalog,
                    &command.source_mod,
                    source,
                    "materials",
                    "material-3d",
                );
            }
        }
        SceneCommand::QueueText3d { command } => {
            crate::app_helpers::register_mod_asset_reference(
                asset_catalog,
                &command.source_mod,
                &command.font,
                "fonts",
                "font-3d",
            );
        }
        SceneCommand::QueueAudioCue { command } => {
            crate::app_helpers::register_mod_asset_reference(
                asset_catalog,
                &command.source_mod,
                &command.clip,
                "audio",
                "audio",
            );
        }
        SceneCommand::QueueUi { command } => {
            ui_support::register_ui_font_asset_references(
                asset_catalog,
                &command.source_mod,
                &command.document,
            );
        }
        _ => {}
    }
}

// Internal migration seam: app-hosted scene hydration remains in this module while
// P0.1 exposes it through `RuntimeSession` lifecycle tracking.
pub(crate) fn queue_scene_document_hydration_for_session(
    session: &RuntimeSession,
    loaded_scene_document: &LoadedSceneDocument,
) -> AmigoResult<()> {
    queue_scene_document_hydration_for_runtime(session.runtime(), loaded_scene_document)?;
    session.scene_session_service().complete_hydration_queue();
    Ok(())
}

fn queue_scene_document_hydration_for_runtime(
    runtime: &Runtime,
    loaded_scene_document: &LoadedSceneDocument,
) -> AmigoResult<()> {
    let scene_command_queue = required::<SceneCommandQueue>(runtime)?;
    let dev_console_state = required::<DevConsoleState>(runtime)?;
    let hydrated_scene_state = required::<HydratedSceneState>(runtime)?;
    let scene_transition_service = required::<SceneTransitionService>(runtime)?;
    if let Ok(asset_catalog) = required::<amigo_assets::AssetCatalog>(runtime) {
        for command in &loaded_scene_document.hydration_plan.commands {
            register_scene_command_asset_references(asset_catalog.as_ref(), command);
        }
    }

    queue_scene_document_hydration(
        scene_command_queue.as_ref(),
        dev_console_state.as_ref(),
        hydrated_scene_state.as_ref(),
        scene_transition_service.as_ref(),
        loaded_scene_document,
    );

    if let Some(scene_session_service) = runtime.resolve::<SceneSessionService>() {
        scene_session_service.complete_hydration_queue();
    }

    Ok(())
}

fn reload_scene_document_for_selected_scene(runtime: &Runtime, scene_id: &str) -> AmigoResult<()> {
    let launch_selection = required::<LaunchSelection>(runtime)?;
    let root_mod = launch_selection.startup_mod.as_deref().ok_or_else(|| {
        AmigoError::Message("cannot load selected scene without startup mod".into())
    })?;
    let request = SceneLoadRequest::new(root_mod, scene_id);
    let scene_session_service = runtime.resolve::<SceneSessionService>();

    if let Some(scene_session_service) = &scene_session_service {
        scene_session_service.begin_scene_load(&request);
    }
    clear_runtime_scene_content_with_runtime(runtime)?;

    match load_scene_document_for_mod(runtime, root_mod, scene_id) {
        Ok(Some(loaded_scene_document)) => {
            if let Some(scene_session_service) = &scene_session_service {
                scene_session_service.complete_scene_load(
                    scene_session_loaded_document_from_loaded(&loaded_scene_document),
                );
            }
            queue_scene_document_hydration_for_runtime(runtime, &loaded_scene_document)
        }
        Ok(None) => {
            if let Some(scene_session_service) = &scene_session_service {
                scene_session_service.fail_scene_load(
                    &request,
                    format!(
                        "scene `{scene_id}` for mod `{root_mod}` did not resolve to a document"
                    ),
                );
            }
            Ok(())
        }
        Err(error) => {
            if let Some(scene_session_service) = &scene_session_service {
                scene_session_service.fail_scene_load(&request, error.to_string());
            }
            Err(error)
        }
    }
}

// Internal migration seam: app-hosted scene command dispatch remains in this module
// while P0.1 exposes it through `RuntimeSession` lifecycle tracking.
pub(crate) fn apply_scene_command_for_session(
    session: &RuntimeSession,
    command: SceneCommand,
) -> AmigoResult<()> {
    if matches!(
        &command,
        SceneCommand::SelectScene { .. } | SceneCommand::ReloadActiveScene
    ) {
        session.scene_session_service().mark_transition_pending();
    }

    apply_scene_command(session.runtime(), command)
}

// Internal migration seam: app-hosted scene cleanup remains in this module while
// P0.1 exposes it through `RuntimeSession` lifecycle tracking.
#[allow(dead_code)]
pub(crate) fn clear_runtime_scene_content_for_session(session: &RuntimeSession) -> AmigoResult<()> {
    session.mark_scene_clearing();

    let result = clear_runtime_scene_content_with_runtime(session.runtime());
    if let Err(error) = &result {
        session
            .scene_session_service()
            .mark_error(format!("scene clear failed: {error}"));
    }

    result
}

#[allow(dead_code)]
pub(crate) fn record_loaded_scene_document_for_runtime(
    runtime: &Runtime,
    loaded_scene_document: &LoadedSceneDocument,
) {
    if let Some(authoring) = runtime.resolve::<amigo_editor_authoring::AuthoringSceneGraphService>()
    {
        authoring.invalidate_scene(
            &loaded_scene_document.summary.source_mod,
            &loaded_scene_document.summary.scene_id,
        );
    }

    let Some(scene_session_service) = runtime.resolve::<SceneSessionService>() else {
        return;
    };

    scene_session_service.apply_loaded_document(scene_session_loaded_document_from_loaded(
        loaded_scene_document,
    ));
}

#[allow(dead_code)]
pub(crate) fn record_scene_hydration_queued_for_runtime(runtime: &Runtime) {
    if let Some(scene_session_service) = runtime.resolve::<SceneSessionService>() {
        scene_session_service.mark_hydration_queued();
    }
}

#[allow(dead_code)]
pub(crate) fn record_scene_lifecycle_error_for_runtime(
    runtime: &Runtime,
    error: impl std::fmt::Display,
) {
    if let Some(scene_session_service) = runtime.resolve::<SceneSessionService>() {
        scene_session_service.mark_error(error.to_string());
    }
}

fn record_scene_command_result_for_runtime(
    runtime: &Runtime,
    command_label: &str,
    result: &AmigoResult<()>,
) {
    let Some(scene_session_service) = runtime.resolve::<SceneSessionService>() else {
        return;
    };

    match result {
        Ok(()) => {
            scene_session_service.mark_scene_command_applied();
        }
        Err(error) => {
            scene_session_service
                .mark_error(format!("scene command `{command_label}` failed: {error}"));
        }
    }
}

fn scene_session_loaded_document_from_loaded(
    loaded_scene_document: &LoadedSceneDocument,
) -> SceneSessionLoadedDocument {
    SceneSessionLoadedDocument::new(
        loaded_scene_document.summary.source_mod.clone(),
        loaded_scene_document.summary.scene_id.clone(),
        loaded_scene_document.summary.relative_path.clone(),
    )
    .with_counts(
        loaded_scene_document.summary.entity_names.len(),
        loaded_scene_document.summary.component_kinds.len(),
        loaded_scene_document.summary.transition_ids.len(),
    )
}

pub(crate) fn apply_scene_command(runtime: &Runtime, command: SceneCommand) -> AmigoResult<()> {
    let selected_scene = match &command {
        SceneCommand::SelectScene { scene } => Some(scene.as_str().to_owned()),
        _ => None,
    };
    if matches!(&command, SceneCommand::ReloadActiveScene) {
        if let (Ok(scene_service), Ok(dev_console_state)) = (
            required::<SceneService>(runtime),
            required::<DevConsoleState>(runtime),
        ) {
            if let Some(active_scene) = scene_service.selected_scene() {
                dev_console_state.write_line(format!(
                    "reloading active scene `{}`",
                    active_scene.as_str()
                ));
            }
        }
    }
    let command_label = amigo_scene::format_scene_command(&command);
    let registry = runtime.required::<amigo_scene::RuntimeSceneCommandHandlerRegistry>()?;
    let result = amigo_runtime::HandlerDispatcher::new(registry)
        .dispatch_first(|handler| {
            handler
                .can_handle(&command)
                .then(|| handler.handle(runtime, command.clone()))
        })
        .unwrap_or_else(|| {
            Err(AmigoError::Message(format!(
                "unhandled scene command in dispatcher: {}",
                command_label
            )))
        });

    record_scene_command_result_for_runtime(runtime, &command_label, &result);
    result?;

    if let Some(scene_id) = selected_scene {
        reload_scene_document_for_selected_scene(runtime, &scene_id)?;
    }

    Ok(())
}

pub(super) fn clear_runtime_scene_content(
    hydrated_scene_state: &HydratedSceneState,
    scene_service: &SceneService,
    dev_console_state: &DevConsoleState,
    sprite_scene_service: &SpriteSceneService,
    layered_image_scene_service: &amigo_runtime_bundles::amigo_2d_layered_image::LayeredImageSceneService,
    render_layer2d_scene_service: &amigo_runtime_bundles::amigo_2d_composition::RenderLayer2dSceneService,
    light_route2d_scene_service: &amigo_runtime_bundles::amigo_2d_composition::LightRoute2dSceneService,
    global_light2d_scene_service: &amigo_runtime_bundles::amigo_2d_lighting::GlobalLight2dSceneService,
    lightmap2d_scene_service: &amigo_runtime_bundles::amigo_2d_lighting::LightMap2dSceneService,
    light_group2d_scene_service: &amigo_runtime_bundles::amigo_2d_lighting::LightGroup2dSceneService,
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
    layered_image_scene_service.clear();
    render_layer2d_scene_service.clear();
    light_route2d_scene_service.clear();
    global_light2d_scene_service.clear();
    lightmap2d_scene_service.clear();
    light_group2d_scene_service.clear();
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
        required::<amigo_runtime_bundles::amigo_2d_layered_image::LayeredImageSceneService>(
            runtime,
        )?
        .as_ref(),
        required::<amigo_runtime_bundles::amigo_2d_composition::RenderLayer2dSceneService>(
            runtime,
        )?
        .as_ref(),
        required::<amigo_runtime_bundles::amigo_2d_composition::LightRoute2dSceneService>(runtime)?
            .as_ref(),
        required::<amigo_runtime_bundles::amigo_2d_lighting::GlobalLight2dSceneService>(runtime)?
            .as_ref(),
        required::<amigo_runtime_bundles::amigo_2d_lighting::LightMap2dSceneService>(runtime)?
            .as_ref(),
        required::<amigo_runtime_bundles::amigo_2d_lighting::LightGroup2dSceneService>(runtime)?
            .as_ref(),
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
    let post_fx_service =
        required::<amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dService>(runtime)?;
    post_fx_service
        .set_scene_stack(amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dStack::default());
    post_fx_service.set_lens_certification_reports(Vec::new());
    post_fx_service.set_renderer_mode("none");

    if let Some(scene_session_service) = runtime.resolve::<SceneSessionService>() {
        scene_session_service.clear_scene_metadata();
    }
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
