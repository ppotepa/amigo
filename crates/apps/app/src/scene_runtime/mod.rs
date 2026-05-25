//! App-side scene command runtime.
//! It loads scene documents, builds hydration plans, and dispatches scene commands into domain services.

use super::*;
use amigo_runtime::EngineSchedulerMode;
use amigo_runtime_control::{RuntimeControlService, build_scene_metadata_for_runtime};
use amigo_scene::{CompiledSceneDocument, SceneDocument, SceneStateValueDocument};
use amigo_scene::{
    SceneFrameClockDocument, SceneFramePresentationDocument, SceneSchedulingDocument,
};
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
    let component_schemas = runtime.resolve::<amigo_scene::ComponentSchemaRegistry>();
    let compiled = amigo_scene::compile_scene_document_from_path_with_component_schemas(
        &document_path,
        &discovered_mod.root_path,
        root_mod,
        component_schemas.as_deref(),
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
    apply_scene_state_defaults(runtime, &document);

    let hydration_plan =
        amigo_scene::build_scene_hydration_plan_for_runtime(runtime, root_mod, &document)
            .map_err(|error| AmigoError::Message(error.to_string()))?;
    let transition_plan = amigo_scene::build_scene_transition_plan(root_mod, &document)
        .map_err(|error| AmigoError::Message(error.to_string()))?;
    let mut runtime_control_metadata =
        build_scene_metadata_for_runtime(runtime, &document, &relative_document_path);
    enrich_runtime_control_metadata_from_authoring(
        &mut runtime_control_metadata,
        root_mod,
        scene_id,
        &discovered_mod.root_path,
        &document_path,
    );

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
        runtime_control_metadata,
    }))
}

fn enrich_runtime_control_metadata_from_authoring(
    metadata: &mut amigo_runtime_control::RuntimeControlSceneMetadata,
    root_mod: &str,
    scene_id: &str,
    mod_root: &Path,
    document_path: &Path,
) {
    let Ok(graph) = amigo_editor_authoring::load_authoring_scene_graph_from_file(
        root_mod.to_owned(),
        scene_id.to_owned(),
        mod_root,
        document_path.to_path_buf(),
    ) else {
        return;
    };

    for node in &graph.nodes {
        apply_authoring_node_to_runtime_metadata(metadata, node);
    }
}

fn apply_authoring_node_to_runtime_metadata(
    metadata: &mut amigo_runtime_control::RuntimeControlSceneMetadata,
    node: &amigo_editor_authoring::AuthoringNode,
) {
    match node.kind {
        amigo_editor_authoring::AuthoringNodeKind::Entity => {
            let scene_object_id = node.semantic.scene_object_id.as_deref();
            let owner_entity_name = node.semantic.owner_entity_name.as_deref();
            for target in metadata.target_lookup.values_mut() {
                if target_matches_authoring_target(target, scene_object_id, owner_entity_name) {
                    target.source_file = Some(node.source_file.display().to_string());
                    target.source_pointer = Some(node.yaml_pointer.clone());
                    if let Some(scene_object_id) = scene_object_id {
                        target.source_id = Some(scene_object_id.to_owned());
                    }
                }
            }
        }
        amigo_editor_authoring::AuthoringNodeKind::Component => {
            let owner_entity_name = node.semantic.owner_entity_name.as_deref();
            let Some(component_type) = node.semantic.component_type.as_deref() else {
                return;
            };
            for target in metadata.target_lookup.values_mut() {
                if !target_matches_authoring_target(target, None, owner_entity_name) {
                    continue;
                }
                let source_file = node.source_file.display().to_string();
                for component in &mut target.components {
                    if component.source_component != component_type
                        && component.console_component != component_type
                    {
                        continue;
                    }
                    component.source_pointer = Some(node.yaml_pointer.clone());
                    target.source_file = Some(source_file.clone());
                    for property in &mut component.properties {
                        property.source_pointer = Some(format!(
                            "{}/{}",
                            node.yaml_pointer,
                            property.property_path.replace('.', "/")
                        ));
                    }
                }
            }
        }
        _ => {}
    }

    for child in &node.children {
        apply_authoring_node_to_runtime_metadata(metadata, child);
    }
}

fn target_matches_authoring_target(
    target: &amigo_runtime_control::RuntimeControlTargetMetadata,
    scene_object_id: Option<&str>,
    owner_entity_name: Option<&str>,
) -> bool {
    scene_object_id.is_some_and(|id| target.source_id.as_deref() == Some(id))
        || owner_entity_name.is_some_and(|name| {
            target.entity_name == name
                || target.display_name == name
                || target.source_id.as_deref() == Some(name)
        })
}

fn apply_scene_state_defaults(runtime: &Runtime, document: &SceneDocument) {
    let Some(state_service) = runtime.resolve::<amigo_state::SceneStateService>() else {
        return;
    };
    let defaults = document
        .state
        .iter()
        .filter_map(|(key, value)| {
            let value = match value {
                SceneStateValueDocument::Bool(value) => amigo_state::SceneStateValue::Bool(*value),
                SceneStateValueDocument::Int(value) => amigo_state::SceneStateValue::Int(*value),
                SceneStateValueDocument::Float(value) => {
                    if !value.is_finite() {
                        return None;
                    }
                    amigo_state::SceneStateValue::Float(*value)
                }
                SceneStateValueDocument::String(value) => {
                    amigo_state::SceneStateValue::String(value.clone())
                }
            };
            Some((key.clone(), value))
        })
        .collect();
    state_service.set_scene_defaults(defaults);
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
    if let Some(clock) = runtime.resolve::<amigo_session::RuntimeFrameClockService>() {
        clock.configure(resolved.frame_clock.clone());
    }
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
    if let Some(frame_clock) = scheduling.frame_clock {
        apply_frame_clock_into_resolved(&mut resolved.frame_clock, frame_clock);
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

fn apply_frame_clock_into_resolved(
    resolved: &mut amigo_session::ResolvedFrameClockConfig,
    document: SceneFrameClockDocument,
) {
    if let Some(strategy) = document
        .strategy
        .as_deref()
        .and_then(parse_frame_clock_strategy)
    {
        resolved.strategy = strategy;
    }
    if let Some(fps) = valid_fps(document.simulation_fps) {
        resolved.simulation_fps = fps;
    }
    if let Some(fps) = valid_fps(document.render_fps) {
        resolved.render_fps = fps;
    }
    if let Some(max) = document.max_catch_up_ticks {
        resolved.max_catch_up_ticks = max.clamp(1, 30);
    }
    if let Some(clamp) = document.clamp_frame_delta_seconds {
        if clamp.is_finite() && clamp > 0.0 {
            resolved.clamp_frame_delta_seconds = clamp.clamp(0.016, 1.0);
        }
    }
    if let Some(presentation) = document.presentation {
        apply_presentation_into_resolved(&mut resolved.presentation, presentation);
    }
}

fn apply_presentation_into_resolved(
    resolved: &mut amigo_session::ResolvedFramePresentationConfig,
    document: SceneFramePresentationDocument,
) {
    if let Some(cache_game_frame) = document.cache_game_frame {
        resolved.cache_game_frame = cache_game_frame;
    }
    if let Some(hold_last_game_frame) = document.hold_last_game_frame {
        resolved.hold_last_game_frame = hold_last_game_frame;
    }
    if let Some(game_ui) = document
        .game_ui
        .as_deref()
        .and_then(parse_presentation_layer_mode)
    {
        resolved.game_ui = game_ui;
    }
    if let Some(devtools) = document.devtools.as_deref() {
        resolved.devtools_live = devtools == "live";
    }
    if let Some(editor) = document.editor.as_deref() {
        resolved.editor_live = editor == "live";
    }
    if let Some(debug_overlay) = document.debug_overlay.as_deref() {
        resolved.debug_overlay_live = debug_overlay == "live";
    }
}

fn parse_frame_clock_strategy(value: &str) -> Option<amigo_session::ResolvedFrameClockStrategy> {
    match value {
        "host_realtime" => Some(amigo_session::ResolvedFrameClockStrategy::HostRealtime),
        "fixed_update_and_render" => {
            Some(amigo_session::ResolvedFrameClockStrategy::FixedUpdateAndRender)
        }
        "fixed_simulation_sampled_render" => {
            Some(amigo_session::ResolvedFrameClockStrategy::FixedSimulationSampledRender)
        }
        "realtime_update_sampled_render" => {
            Some(amigo_session::ResolvedFrameClockStrategy::RealtimeUpdateSampledRender)
        }
        _ => None,
    }
}

fn parse_presentation_layer_mode(
    value: &str,
) -> Option<amigo_session::ResolvedPresentationLayerMode> {
    match value {
        "cached" => Some(amigo_session::ResolvedPresentationLayerMode::Cached),
        "live" => Some(amigo_session::ResolvedPresentationLayerMode::Live),
        _ => None,
    }
}

fn valid_fps(value: Option<f32>) -> Option<f32> {
    value
        .filter(|fps| fps.is_finite() && *fps > 0.0)
        .map(|fps| fps.clamp(1.0, 240.0))
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
    runtime_control_service: &RuntimeControlService,
    loaded_scene_document: &LoadedSceneDocument,
) {
    runtime_control_service
        .replace_scene_metadata(loaded_scene_document.runtime_control_metadata.clone());
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
        SceneCommand::Plugin { command } => {
            if let Some(command) = command.payload_as::<amigo_scene::Sprite2dSceneCommand>() {
                crate::app_helpers::register_mod_asset_reference(
                    asset_catalog,
                    &command.source_mod,
                    &command.texture,
                    "spritesheets",
                    "sprite-sheet-2d",
                );
            } else if let Some(command) = command.payload_as::<amigo_scene::Text2dSceneCommand>() {
                crate::app_helpers::register_mod_asset_reference(
                    asset_catalog,
                    &command.source_mod,
                    &command.font,
                    "fonts",
                    "font-2d",
                );
            } else if let Some(command) = command.payload_as::<amigo_scene::Mesh3dSceneCommand>() {
                crate::app_helpers::register_mod_asset_reference(
                    asset_catalog,
                    &command.source_mod,
                    &command.mesh_asset,
                    "meshes",
                    "mesh-3d",
                );
            } else if let Some(command) =
                command.payload_as::<amigo_scene::Material3dSceneCommand>()
            {
                if let Some(source) = &command.source {
                    crate::app_helpers::register_mod_asset_reference(
                        asset_catalog,
                        &command.source_mod,
                        source,
                        "materials",
                        "material-3d",
                    );
                }
            } else if let Some(command) = command.payload_as::<amigo_scene::Text3dSceneCommand>() {
                crate::app_helpers::register_mod_asset_reference(
                    asset_catalog,
                    &command.source_mod,
                    &command.font,
                    "fonts",
                    "font-3d",
                );
            } else if let Some(command) = command.payload_as::<amigo_scene::LayeredImage2dSceneCommand>() {
                crate::app_helpers::register_mod_asset_reference(
                    asset_catalog,
                    &command.source_mod,
                    &command.asset,
                    "layered-images",
                    "layered-image-2d",
                );
            } else if let Some(command) = command.payload_as::<amigo_scene::TileMap2dSceneCommand>() {
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
            } else if let Some(command) = command.payload_as::<amigo_scene::Sprite2dSceneCommand>() {
                crate::app_helpers::register_mod_asset_reference(
                    asset_catalog,
                    &command.source_mod,
                    &command.texture,
                    "spritesheets",
                    "sprite-sheet-2d",
                );
            } else if let Some(command) = command.payload_as::<amigo_scene::Text2dSceneCommand>() {
                crate::app_helpers::register_mod_asset_reference(
                    asset_catalog,
                    &command.source_mod,
                    &command.font,
                    "fonts",
                    "font-2d",
                );
            } else if let Some(command) = command.payload_as::<amigo_scene::AudioCueSceneCommand>()
            {
                crate::app_helpers::register_mod_asset_reference(
                    asset_catalog,
                    &command.source_mod,
                    &command.clip,
                    "audio",
                    "audio",
                );
            } else if let Some(command) = command.payload_as::<amigo_scene::UiSceneCommand>() {
                ui_support::register_ui_font_asset_references(
                    asset_catalog,
                    &command.source_mod,
                    &command.document,
                );
            }
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
    let runtime_control_service = required::<RuntimeControlService>(runtime)?;
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
        runtime_control_service.as_ref(),
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
    if matches!(&command, SceneCommand::Plugin { .. }) {
        let plugin_registry = runtime.required::<amigo_scene::ScenePluginCommandHandlerRegistry>()?;
        let result = plugin_registry
            .dispatch(runtime, command.clone())
            .map(|handled| handled.unwrap_or(()));
        record_scene_command_result_for_runtime(runtime, &command_label, &result);
        result?;
        return Ok(());
    }

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

pub(super) fn clear_runtime_scene_content(runtime: &Runtime) -> AmigoResult<()> {
    let hydrated_scene_state = required::<HydratedSceneState>(runtime)?;
    let scene_service = required::<SceneService>(runtime)?;
    let runtime_control_service = required::<RuntimeControlService>(runtime)?;
    let dev_console_state = required::<DevConsoleState>(runtime)?;

    let previous = hydrated_scene_state.clear();
    runtime_control_service.clear_scene_metadata();

    if !previous.entity_names.is_empty() {
        let removed = scene_service.remove_entities_by_name(&previous.entity_names);
        dev_console_state.write_line(format!(
            "removed {removed} hydrated scene entities from `{}`",
            previous.scene_id.as_deref().unwrap_or("unknown")
        ));
    }

    if let Some(reset_registry) = runtime.resolve::<amigo_scene::SceneResetHandlerRegistry>() {
        reset_registry.reset_all(runtime)?;
    }

    Ok(())
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

    clear_runtime_scene_content(runtime)?;

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
