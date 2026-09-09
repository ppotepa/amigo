use amigo_core::AmigoResult;
use amigo_editor_authoring::AuthoringSceneGraphService;
use amigo_input_api::{InputEvent, InputModifiers, KeyCode, MouseButton};
use amigo_overlay_api::UiViewportSize;
use amigo_runtime::Runtime;
use amigo_ui::UiInputViewportState;

use crate::layout::{inspector_dock_panel, EditorLayout, EditorPanelKind};
use crate::runtime_apply::{apply_property_request, ApplyRequest, ApplyResult};
use crate::selection::{select_node_by_id, select_viewport_target};
use crate::state::{
    EditorHitAction, EditorHitTarget, EditorOpenMode, EditorPropertyValue, EditorSourceScalar,
    EditorTreeMode, IngameEditorState, SelectionSource,
};

pub fn handle_editor_input(
    runtime: &Runtime,
    event: &InputEvent,
    modifiers: InputModifiers,
) -> AmigoResult<bool> {
    let Some(state) = runtime.resolve::<IngameEditorState>() else {
        return Ok(false);
    };

    if !state.enabled() {
        return Ok(false);
    }

    let snapshot = state.snapshot();
    let open_mode = snapshot.open_mode;

    match event {
        InputEvent::Key {
            key: KeyCode::F3,
            pressed: true,
        } => {
            if state.is_open() && open_mode == EditorOpenMode::Full {
                state.set_open(false);
            } else {
                state.open_full_editor();
            }
            Ok(true)
        }
        InputEvent::Key {
            key: KeyCode::E,
            pressed: true,
        } if modifiers.control || modifiers.super_key => {
            if state.is_open() && open_mode == EditorOpenMode::Full {
                state.set_open(false);
            } else {
                state.open_full_editor();
            }
            Ok(true)
        }
        InputEvent::Key {
            key: KeyCode::Escape,
            pressed: true,
        } if state.is_open() && open_mode == EditorOpenMode::InspectorDock => {
            state.close_inspector_dock();
            Ok(true)
        }
        InputEvent::CursorMoved { x, y } => {
            state.set_cursor(*x as f32, *y as f32);
            state.update_viewport_pan(*x as f32, *y as f32);
            if !state.is_open() {
                return Ok(false);
            }
            if open_mode == EditorOpenMode::InspectorDock {
                return Ok(false);
            }
            Ok(true)
        }
        InputEvent::MouseButton {
            button: MouseButton::Middle,
            pressed: true,
        } if state.is_open() => {
            let snapshot = state.snapshot();
            let Some((x, y)) = snapshot.cursor else {
                return Ok(true);
            };
            let layout = current_editor_layout(runtime);
            if matches!(layout.panel_for_point(x, y), EditorPanelKind::Viewport) {
                state.begin_viewport_pan(x, y);
            }
            Ok(true)
        }
        InputEvent::MouseButton {
            button: MouseButton::Middle,
            pressed: false,
        } if state.is_open() => {
            state.end_viewport_pan();
            Ok(true)
        }
        InputEvent::MouseButton {
            button: MouseButton::Left,
            pressed: true,
        } if state.is_open() => {
            let snapshot = state.snapshot();
            let Some((x, y)) = snapshot.cursor else {
                return Ok(true);
            };

            if open_mode == EditorOpenMode::InspectorDock {
                if !is_inside_inspector_dock(runtime, x, y) {
                    return Ok(false);
                }
                if let Some(target) = state.hit_target_at(x, y) {
                    handle_hit_target(runtime, state.as_ref(), target, x)?;
                }
                return Ok(true);
            }

            if let Some(target) = state.hit_target_at(x, y) {
                handle_hit_target(runtime, state.as_ref(), target, x)?;
            } else {
                let layout = current_editor_layout(runtime);
                if matches!(layout.panel_for_point(x, y), EditorPanelKind::Viewport) {
                    let snapshot = state.snapshot();
                    if let Some((logical_x, logical_y)) =
                        layout.game_viewport_layout().screen_to_logical_with_view(
                            x,
                            y,
                            snapshot.viewport_pan_x,
                            snapshot.viewport_pan_y,
                            snapshot.viewport_zoom,
                        )
                    {
                        if let Some(service) = runtime.resolve::<AuthoringSceneGraphService>() {
                            if let Ok(graph) = service.graph_for_current_scene(runtime) {
                                select_viewport_target(
                                    runtime,
                                    state.as_ref(),
                                    &graph,
                                    logical_x,
                                    logical_y,
                                );
                            }
                        }
                    } else {
                        state.clear_selection();
                    }
                }
            }
            Ok(true)
        }
        InputEvent::MouseWheel { delta_y } if state.is_open() => {
            let snapshot = state.snapshot();
            let Some((x, y)) = snapshot.cursor else {
                return Ok(true);
            };

            if open_mode == EditorOpenMode::InspectorDock {
                if !is_inside_inspector_dock(runtime, x, y) {
                    return Ok(false);
                }
                state.scroll_properties(-*delta_y * 24.0);
                return Ok(true);
            }

            let layout = current_editor_layout(runtime);
            match layout.panel_for_point(x, y) {
                EditorPanelKind::Tree => state.scroll_tree(-*delta_y * 24.0),
                EditorPanelKind::Properties => state.scroll_properties(-*delta_y * 24.0),
                _ => {}
            }

            Ok(true)
        }
        InputEvent::MouseButton { .. } if state.is_open() => {
            if open_mode == EditorOpenMode::InspectorDock {
                let snapshot = state.snapshot();
                if let Some((x, y)) = snapshot.cursor {
                    return Ok(is_inside_inspector_dock(runtime, x, y));
                }
                return Ok(false);
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn is_inside_inspector_dock(runtime: &Runtime, x: f32, y: f32) -> bool {
    let viewport = runtime
        .resolve::<UiInputViewportState>()
        .and_then(|viewport| viewport.get())
        .unwrap_or_else(|| UiViewportSize::new(1280.0, 720.0));
    inspector_dock_panel(viewport).rect.contains(x, y)
}

fn handle_hit_target(
    runtime: &Runtime,
    state: &IngameEditorState,
    target: EditorHitTarget,
    cursor_x: f32,
) -> AmigoResult<()> {
    match target.action {
        EditorHitAction::SelectNode {
            node_id,
            source_path,
            yaml_pointer,
        } => {
            if let Some(service) = runtime.resolve::<AuthoringSceneGraphService>() {
                if let Ok(graph) = service.graph_for_current_scene(runtime) {
                    select_node_by_id(
                        runtime,
                        state,
                        &graph,
                        node_id,
                        source_path,
                        yaml_pointer,
                        SelectionSource::Tree,
                    );
                } else {
                    state.set_status("selection failed: authoring graph unavailable".to_owned());
                }
            } else {
                state.set_status("selection failed: authoring service unavailable".to_owned());
            }
        }
        EditorHitAction::Slider {
            property_id,
            target: runtime_target,
            source,
            min,
            max,
            current,
        } => {
            let value = slider_value_from_cursor(cursor_x, target.rect, min, max);
            let result = apply_property_request(
                runtime,
                state,
                ApplyRequest {
                    property_id: &property_id,
                    target: runtime_target.as_ref(),
                    previous: Some(EditorPropertyValue::Number(current)),
                    next: EditorPropertyValue::Number(value),
                },
            )?;
            stage_source_scalar_patch(
                state,
                &result,
                source.as_ref(),
                &EditorPropertyValue::Number(value),
            );
            state.set_status(format!(
                "{property_id}: {current:.3} -> {value:.3} {result:?}"
            ));
        }
        EditorHitAction::Toggle {
            property_id,
            target: runtime_target,
            source,
            current,
        } => {
            let previous = match state.override_value(&property_id) {
                Some(EditorPropertyValue::Bool(value)) => value,
                _ => current,
            };
            let next = !previous;
            let result = apply_property_request(
                runtime,
                state,
                ApplyRequest {
                    property_id: &property_id,
                    target: runtime_target.as_ref(),
                    previous: Some(EditorPropertyValue::Bool(previous)),
                    next: EditorPropertyValue::Bool(next),
                },
            )?;
            stage_source_scalar_patch(
                state,
                &result,
                source.as_ref(),
                &EditorPropertyValue::Bool(next),
            );
            state.set_status(format!("{property_id}: {previous} -> {next} {result:?}"));
        }
        EditorHitAction::TextCommit {
            property_id,
            target: runtime_target,
            value,
        } => {
            let result = apply_property_request(
                runtime,
                state,
                ApplyRequest {
                    property_id: &property_id,
                    target: runtime_target.as_ref(),
                    previous: None,
                    next: EditorPropertyValue::Text(value.clone()),
                },
            )?;
            state.set_status(format!("{property_id}: {value} {result:?}"));
        }
        EditorHitAction::EnumSelect {
            property_id,
            target: runtime_target,
            source,
            value,
        } => {
            let result = apply_property_request(
                runtime,
                state,
                ApplyRequest {
                    property_id: &property_id,
                    target: runtime_target.as_ref(),
                    previous: None,
                    next: EditorPropertyValue::Enum(value.clone()),
                },
            )?;
            stage_source_scalar_patch(
                state,
                &result,
                source.as_ref(),
                &EditorPropertyValue::Enum(value.clone()),
            );
            state.set_status(format!("{property_id}: {value} {result:?}"));
        }
        EditorHitAction::ColorCommit {
            property_id,
            target: runtime_target,
            value,
        } => {
            let result = apply_property_request(
                runtime,
                state,
                ApplyRequest {
                    property_id: &property_id,
                    target: runtime_target.as_ref(),
                    previous: None,
                    next: EditorPropertyValue::Color(value.clone()),
                },
            )?;
            state.set_status(format!("{property_id}: {value} {result:?}"));
        }
        EditorHitAction::NumberCommit {
            property_id,
            target: runtime_target,
            source,
            value,
        } => {
            let result = apply_property_request(
                runtime,
                state,
                ApplyRequest {
                    property_id: &property_id,
                    target: runtime_target.as_ref(),
                    previous: None,
                    next: EditorPropertyValue::Number(value),
                },
            )?;
            stage_source_scalar_patch(
                state,
                &result,
                source.as_ref(),
                &EditorPropertyValue::Number(value),
            );
            state.set_status(format!("{property_id}: -> {value:.3} {result:?}"));
        }
        EditorHitAction::Vec2Commit {
            property_id,
            target: runtime_target,
            x,
            y,
        } => {
            let result = apply_property_request(
                runtime,
                state,
                ApplyRequest {
                    property_id: &property_id,
                    target: runtime_target.as_ref(),
                    previous: None,
                    next: EditorPropertyValue::Vec2(x, y),
                },
            )?;
            state.set_status(format!("{property_id}: ({x:.3}, {y:.3}) {result:?}"));
        }
        EditorHitAction::AssetPick {
            property_id,
            target: runtime_target,
            asset,
        } => {
            let result = apply_property_request(
                runtime,
                state,
                ApplyRequest {
                    property_id: &property_id,
                    target: runtime_target.as_ref(),
                    previous: None,
                    next: EditorPropertyValue::AssetRef(asset.clone()),
                },
            )?;
            state.set_status(format!("{property_id}: {asset} {result:?}"));
        }
        EditorHitAction::Command { command } => match command.as_str() {
            "editor.toggle" => state.toggle(),
            "editor.tree.scene" => state.set_tree_mode(EditorTreeMode::Scene),
            "editor.tree.clean" => state.set_tree_mode(EditorTreeMode::Scene),
            "editor.tree.stack" => state.set_tree_mode(EditorTreeMode::Stack),
            "editor.tree.raw" => state.set_tree_mode(EditorTreeMode::RawYaml),
            "editor.save_source_edit" => {
                let patches = state.pending_source_scalar_patches();
                if patches.is_empty() {
                    state.set_status("source save: no pending scalar edit".to_owned());
                    return Ok(());
                }
                let Some(service) = runtime.resolve::<AuthoringSceneGraphService>() else {
                    state
                        .set_status("source save failed: authoring service unavailable".to_owned());
                    return Ok(());
                };
                match service.apply_source_scalar_patches(runtime, &patches) {
                    Ok(()) => {
                        state.clear_pending_source_scalar_patch();
                        state.set_status("source save: persisted".to_owned());
                    }
                    Err(error) => state.set_status(format!("source save failed: {error}")),
                }
            }
            "editor.discard_source_edit" => {
                state.clear_pending_source_scalar_patch();
                state.set_status(
                    "source edit draft discarded; runtime value is unchanged".to_owned(),
                );
            }
            _ => state.set_status(format!("unknown editor command: {command}")),
        },
        EditorHitAction::ToggleTreeNode { node_id } => {
            state.toggle_node_collapsed(&node_id);
        }
        EditorHitAction::ConsumeOnly => {}
    }

    Ok(())
}

fn stage_source_scalar_patch(
    state: &IngameEditorState,
    result: &ApplyResult,
    source: Option<&EditorSourceScalar>,
    next: &EditorPropertyValue,
) {
    if !matches!(result, ApplyResult::Applied) {
        return;
    }
    if let Some(patch) = source.and_then(|source| source.patch_for(next)) {
        state.stage_source_scalar_patch(patch);
    }
}

fn slider_value_from_cursor(
    cursor_x: f32,
    rect: crate::state::EditorRect,
    min: f32,
    max: f32,
) -> f32 {
    let t = if rect.width <= 0.0 {
        0.0
    } else {
        ((cursor_x - rect.x) / rect.width).clamp(0.0, 1.0)
    };
    min + (max - min) * t
}

fn current_editor_layout(runtime: &Runtime) -> EditorLayout {
    let viewport = runtime
        .resolve::<UiInputViewportState>()
        .and_then(|viewport| viewport.get())
        .unwrap_or_else(|| UiViewportSize::new(1280.0, 720.0));
    EditorLayout::new(viewport)
}
