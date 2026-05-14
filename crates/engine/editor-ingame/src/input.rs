use amigo_core::AmigoResult;
use amigo_editor_authoring::AuthoringSceneGraphService;
use amigo_input_api::{InputEvent, InputModifiers, KeyCode, MouseButton};
use amigo_render_wgpu::UiViewportSize;
use amigo_runtime::Runtime;
use amigo_ui::UiInputViewportState;

use crate::layout::{EditorLayout, EditorPanelKind};
use crate::runtime_apply::apply_property_value;
use crate::selection::select_viewport_target;
use crate::state::{EditorHitAction, EditorHitTarget, EditorPropertyValue, IngameEditorState};

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

    match event {
        InputEvent::Key {
            key: KeyCode::F3,
            pressed: true,
        } => {
            state.toggle();
            Ok(true)
        }
        InputEvent::Key {
            key: KeyCode::E,
            pressed: true,
        } if modifiers.control || modifiers.super_key => {
            state.toggle();
            Ok(true)
        }
        InputEvent::CursorMoved { x, y } => {
            state.set_cursor(*x as f32, *y as f32);
            state.update_viewport_pan(*x as f32, *y as f32);
            Ok(state.is_open())
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
                                    state.as_ref(),
                                    &graph,
                                    logical_x,
                                    logical_y,
                                );
                            }
                        }
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

            let layout = current_editor_layout(runtime);
            match layout.panel_for_point(x, y) {
                EditorPanelKind::Tree => state.scroll_tree(-*delta_y * 24.0),
                EditorPanelKind::Properties => state.scroll_properties(-*delta_y * 24.0),
                _ => {}
            }

            Ok(true)
        }
        InputEvent::MouseButton { .. } if state.is_open() => Ok(true),
        _ => Ok(false),
    }
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
            state.select_node(node_id, source_path, yaml_pointer);
        }
        EditorHitAction::Slider {
            property_id,
            target: runtime_target,
            min,
            max,
        } => {
            let value = slider_value_from_cursor(cursor_x, target.rect, min, max);
            apply_property_value(
                runtime,
                state,
                &property_id,
                runtime_target.as_ref(),
                EditorPropertyValue::Number(value),
            )?;
            state.set_status(format!("{property_id} = {value:.3}"));
        }
        EditorHitAction::Toggle {
            property_id,
            target: runtime_target,
            current,
        } => {
            let previous = match state.override_value(&property_id) {
                Some(EditorPropertyValue::Bool(value)) => value,
                _ => current,
            };
            let next = !previous;
            apply_property_value(
                runtime,
                state,
                &property_id,
                runtime_target.as_ref(),
                EditorPropertyValue::Bool(next),
            )?;
            state.set_status(format!("{property_id} = {next}"));
        }
        EditorHitAction::Command { command } => {
            if command == "editor.toggle" {
                state.toggle();
            }
        }
        EditorHitAction::ToggleTreeNode { node_id } => {
            state.toggle_node_collapsed(&node_id);
        }
        EditorHitAction::ConsumeOnly => {}
    }

    Ok(())
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
