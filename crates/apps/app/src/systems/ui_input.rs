use super::super::*;
use crate::runtime_context::RuntimeContext;
use amigo_math::ColorRgba;
use amigo_render_wgpu::UiRect;
use amigo_ui::{UiCurvePoint, curve_editor_edit_from_mouse};

pub(crate) fn process_ui_input(runtime: &Runtime) -> AmigoResult<()> {
    let viewport = RuntimeContext::new(runtime)
        .required::<super::UiInputViewportState>()?
        .get();
    let Some(viewport) = viewport else {
        return Ok(());
    };

    let ctx = RuntimeContext::new(runtime);
    let ui_input = ctx.required::<UiInputService>()?;
    let snapshot = ui_input.snapshot();
    if !snapshot.mouse_left_released
        && !snapshot.mouse_left_down
        && snapshot.mouse_wheel_y.abs() <= f32::EPSILON
    {
        return Ok(());
    }

    let Some(mouse_position) = snapshot.mouse_position else {
        return Ok(());
    };

    let ui_scene = ctx.required::<UiSceneService>()?;
    let ui_state = ctx.required::<UiStateService>()?;
    let ui_theme = ctx.required::<UiThemeService>()?;
    let script_event_queue = ctx.required::<ScriptEventQueue>()?;
    let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
        ui_scene.as_ref(),
        ui_state.as_ref(),
        ui_theme.as_ref(),
    );
    if snapshot.mouse_left_released {
        if let Some(active_path) = ui_input.active_path() {
            ui_state.clear_background(&active_path);
            ui_input.set_active_path(None);
        }
    }
    for document in resolved.iter().rev() {
        let layout = build_ui_layout_tree(viewport, &document.overlay);
        let Some(path) =
            crate::ui_runtime::hit_test_ui_layout(&layout, mouse_position.x, mouse_position.y)
        else {
            continue;
        };

        if !ui_state.is_enabled(&path) {
            continue;
        }

        let Some(layout_node) = crate::ui_runtime::find_ui_layout_node(&layout, &path) else {
            continue;
        };

        if snapshot.mouse_left_down {
            if is_interactive_node(&layout_node.node.kind)
                || document.click_bindings.contains_key(&path)
                || document.change_bindings.contains_key(&path)
            {
                ui_input.set_active_path(Some(path.clone()));
                ui_state.set_background(path.clone(), pressed_background(layout_node));
            }
            if let UiOverlayNodeKind::Slider { min, max, step, .. } = layout_node.node.kind {
                let value =
                    slider_value_from_mouse(layout_node.rect, mouse_position.x, min, max, step);
                if ui_state.set_value(path.clone(), value) {
                    if let Some(binding) = document.change_bindings.get(&path) {
                        publish_ui_binding(script_event_queue.as_ref(), binding, Some(value));
                    }
                }
                break;
            }
            if let UiOverlayNodeKind::ColorPickerRgb { color } = layout_node.node.kind {
                let color = color_picker_rgb_color_from_mouse(
                    layout_node.rect,
                    mouse_position.x,
                    mouse_position.y,
                    color,
                );
                if ui_state.set_background(path.clone(), color) {
                    if let Some(binding) = document.change_bindings.get(&path) {
                        publish_ui_binding_with_payload(
                            script_event_queue.as_ref(),
                            binding,
                            vec![
                                format!("{:.4}", color.r),
                                format!("{:.4}", color.g),
                                format!("{:.4}", color.b),
                            ],
                        );
                    }
                }
                break;
            }
            if let UiOverlayNodeKind::CurveEditor { points } = &layout_node.node.kind {
                let edit_rect = amigo_ui::UiRect::new(
                    layout_node.rect.x,
                    layout_node.rect.y,
                    layout_node.rect.width,
                    layout_node.rect.height,
                )
                .inset(10.0);
                let edit = curve_editor_edit_from_mouse(
                    edit_rect,
                    &points
                        .iter()
                        .map(|point| UiCurvePoint::new(point.t, point.value))
                        .collect::<Vec<_>>(),
                    mouse_position.x,
                    mouse_position.y,
                );
                if let Some(edit) = edit {
                    if ui_state.set_curve_points(path.clone(), edit.points.clone()) {
                        if let Some(binding) = document.change_bindings.get(&path) {
                            publish_ui_binding_with_payload(
                                script_event_queue.as_ref(),
                                binding,
                                edit.payload(),
                            );
                        }
                    }
                }
                break;
            }
        }

        if let UiOverlayNodeKind::Dropdown {
            options,
            expanded,
            scroll_offset,
            ..
        } = &layout_node.node.kind
        {
            let effective_scroll_offset =
                ui_state.dropdown_scroll_offset(&path).max(*scroll_offset);
            if snapshot.mouse_wheel_y.abs() > f32::EPSILON && *expanded {
                let visible_count = crate::ui_runtime::dropdown_visible_option_count(options.len());
                let next = effective_scroll_offset - snapshot.mouse_wheel_y * 0.65;
                ui_state.set_dropdown_scroll_offset(
                    path.clone(),
                    next,
                    options.len(),
                    visible_count,
                );
                break;
            }

            if snapshot.mouse_left_down
                && *expanded
                && dropdown_scrollbar_contains(
                    layout_node.rect,
                    options.len(),
                    mouse_position.x,
                    mouse_position.y,
                )
            {
                let visible_count = crate::ui_runtime::dropdown_visible_option_count(options.len());
                let next = dropdown_scroll_offset_from_mouse(
                    layout_node.rect,
                    options.len(),
                    visible_count,
                    mouse_position.y,
                );
                ui_state.set_dropdown_scroll_offset(
                    path.clone(),
                    next,
                    options.len(),
                    visible_count,
                );
                break;
            }

            if !snapshot.mouse_left_released {
                break;
            }

            if !expanded {
                ui_state.set_expanded(path.clone(), true);
                break;
            }

            if dropdown_scrollbar_contains(
                layout_node.rect,
                options.len(),
                mouse_position.x,
                mouse_position.y,
            ) {
                let visible_count = crate::ui_runtime::dropdown_visible_option_count(options.len());
                let next = dropdown_scroll_offset_from_mouse(
                    layout_node.rect,
                    options.len(),
                    visible_count,
                    mouse_position.y,
                );
                ui_state.set_dropdown_scroll_offset(
                    path.clone(),
                    next,
                    options.len(),
                    visible_count,
                );
                break;
            }

            let Some(index) = dropdown_option_index_from_mouse(
                layout_node.rect,
                mouse_position.y,
                effective_scroll_offset,
            ) else {
                ui_state.set_expanded(path.clone(), false);
                break;
            };
            if let Some(selected) = options.get(index).cloned() {
                ui_state.set_selected(path.clone(), selected.clone());
                ui_state.set_expanded(path.clone(), false);
                if let Some(binding) = document.change_bindings.get(&path) {
                    publish_ui_binding_with_payload(
                        script_event_queue.as_ref(),
                        binding,
                        vec![selected],
                    );
                }
            }
            break;
        }

        if !snapshot.mouse_left_released {
            continue;
        }

        if let UiOverlayNodeKind::Toggle { checked, .. } = layout_node.node.kind {
            let value = if checked { 0.0 } else { 1.0 };
            ui_state.set_value(path.clone(), value);
            if let Some(binding) = document.change_bindings.get(&path) {
                publish_ui_binding(script_event_queue.as_ref(), binding, Some(value));
            }
            break;
        }

        if let UiOverlayNodeKind::OptionSet { options, .. } = &layout_node.node.kind {
            if let Some(selected) =
                option_set_value_from_mouse(layout_node.rect, options, mouse_position.x)
            {
                ui_state.set_selected(path.clone(), selected.clone());
                if let Some(binding) = document.change_bindings.get(&path) {
                    publish_ui_binding_with_payload(
                        script_event_queue.as_ref(),
                        binding,
                        vec![selected],
                    );
                }
            }
            break;
        }

        if let UiOverlayNodeKind::TabView { tabs, .. } = &layout_node.node.kind {
            if let Some(selected) = amigo_render_wgpu::tab_view_tab_from_mouse(
                layout_node.rect,
                &layout_node.node,
                tabs,
                mouse_position.x,
                mouse_position.y,
            ) {
                ui_state.set_selected(path.clone(), selected.clone());
                if let Some(binding) = document.change_bindings.get(&path) {
                    publish_ui_binding_with_payload(
                        script_event_queue.as_ref(),
                        binding,
                        vec![selected],
                    );
                }
            }
            break;
        }

        if let Some(binding) = document.click_bindings.get(&path) {
            script_event_queue.publish(ScriptEvent::new(
                binding.event.clone(),
                binding.payload.clone(),
            ));
            break;
        }
    }

    Ok(())
}

fn is_interactive_node(kind: &UiOverlayNodeKind) -> bool {
    matches!(
        kind,
        UiOverlayNodeKind::Button { .. }
            | UiOverlayNodeKind::Slider { .. }
            | UiOverlayNodeKind::Toggle { .. }
            | UiOverlayNodeKind::OptionSet { .. }
            | UiOverlayNodeKind::TabView { .. }
            | UiOverlayNodeKind::Dropdown { .. }
            | UiOverlayNodeKind::ColorPickerRgb { .. }
            | UiOverlayNodeKind::CurveEditor { .. }
    )
}

fn pressed_background(node: &OverlayUiLayoutNode) -> ColorRgba {
    let base = node
        .node
        .style
        .background
        .unwrap_or(ColorRgba::new(0.2, 0.33, 0.66, 1.0));
    ColorRgba::new(
        (base.r * 0.68).clamp(0.0, 1.0),
        (base.g * 0.68).clamp(0.0, 1.0),
        (base.b * 0.68).clamp(0.0, 1.0),
        base.a,
    )
}

fn slider_value_from_mouse(rect: UiRect, mouse_x: f32, min: f32, max: f32, step: f32) -> f32 {
    if rect.width <= f32::EPSILON {
        return 0.0;
    }
    let mut value = ((mouse_x - rect.x) / rect.width).clamp(0.0, 1.0);
    let range = (max - min).abs();
    if step > f32::EPSILON && range > f32::EPSILON {
        let normalized_step = (step / range).clamp(0.0, 1.0);
        if normalized_step > f32::EPSILON {
            value = (value / normalized_step).round() * normalized_step;
        }
    }
    value.clamp(0.0, 1.0)
}

fn publish_ui_binding(
    script_event_queue: &ScriptEventQueue,
    binding: &UiEventBinding,
    value: Option<f32>,
) {
    let mut payload = binding.payload.clone();
    if let Some(value) = value {
        payload.push(format!("{value:.4}"));
    }
    script_event_queue.publish(ScriptEvent::new(binding.event.clone(), payload));
}

fn publish_ui_binding_with_payload(
    script_event_queue: &ScriptEventQueue,
    binding: &UiEventBinding,
    extra_payload: Vec<String>,
) {
    let mut payload = binding.payload.clone();
    payload.extend(extra_payload);
    script_event_queue.publish(ScriptEvent::new(binding.event.clone(), payload));
}

fn option_set_value_from_mouse(rect: UiRect, options: &[String], mouse_x: f32) -> Option<String> {
    if rect.width <= f32::EPSILON || options.is_empty() {
        return None;
    }
    let normalized = ((mouse_x - rect.x) / rect.width).clamp(0.0, 0.999_999);
    let index = (normalized * options.len() as f32).floor() as usize;
    options.get(index).cloned()
}

fn dropdown_option_index_from_mouse(
    rect: UiRect,
    mouse_y: f32,
    scroll_offset: f32,
) -> Option<usize> {
    let row_height = 38.0_f32.min(rect.height.max(0.0));
    if row_height <= f32::EPSILON {
        return None;
    }

    let row = (mouse_y - rect.y) / row_height;
    if row < 1.0 {
        return None;
    }

    let option_index = (scroll_offset + row - 1.0).floor();
    if option_index.is_finite() && option_index >= 0.0 {
        Some(option_index as usize)
    } else {
        None
    }
}

fn dropdown_scrollbar_contains(
    rect: UiRect,
    option_count: usize,
    mouse_x: f32,
    mouse_y: f32,
) -> bool {
    let visible_count = crate::ui_runtime::dropdown_visible_option_count(option_count);
    if option_count <= visible_count {
        return false;
    }
    let row_height = 38.0_f32.min(rect.height.max(0.0));
    let scrollbar_width = 10.0_f32.min(rect.width.max(0.0));
    mouse_x >= rect.x + rect.width - scrollbar_width
        && mouse_x <= rect.x + rect.width
        && mouse_y >= rect.y + row_height
        && mouse_y <= rect.y + row_height * (visible_count as f32 + 1.0)
}

fn dropdown_scroll_offset_from_mouse(
    rect: UiRect,
    option_count: usize,
    visible_count: usize,
    mouse_y: f32,
) -> f32 {
    let row_height = 38.0_f32.min(rect.height.max(0.0));
    let track_y = rect.y + row_height;
    let track_height = row_height * visible_count as f32;
    if track_height <= f32::EPSILON || option_count <= visible_count {
        return 0.0;
    }
    let visible_ratio = (visible_count as f32 / option_count as f32).clamp(0.05, 1.0);
    let thumb_height = (track_height * visible_ratio).clamp(18.0, track_height);
    let travel = (track_height - thumb_height).max(1.0);
    let relative = ((mouse_y - track_y - thumb_height * 0.5) / travel).clamp(0.0, 1.0);
    relative * option_count.saturating_sub(visible_count) as f32
}

fn color_picker_rgb_color_from_mouse(
    rect: UiRect,
    mouse_x: f32,
    mouse_y: f32,
    current: ColorRgba,
) -> ColorRgba {
    let padding = 8.0;
    let swatch_width = 54.0_f32.min((rect.width - padding * 2.0).max(0.0));
    let slider_x = rect.x + padding + swatch_width + 10.0 + 24.0;
    let slider_width = (rect.x + rect.width - padding - slider_x).max(0.0);
    if slider_width <= f32::EPSILON {
        return current;
    }

    let slider_height = 22.0;
    let row_stride = slider_height + 10.0;
    let relative_y = mouse_y - rect.y - padding;
    let row = (relative_y / row_stride).floor() as i32;
    if !(0..=2).contains(&row) {
        return current;
    }

    let value = ((mouse_x - slider_x) / slider_width).clamp(0.0, 1.0);
    match row {
        0 => ColorRgba::new(value, current.g, current.b, current.a),
        1 => ColorRgba::new(current.r, value, current.b, current.a),
        2 => ColorRgba::new(current.r, current.g, value, current.a),
        _ => current,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropdown_option_index_uses_fractional_scroll_offset() {
        let rect = UiRect::new(100.0, 40.0, 250.0, 38.0);

        assert_eq!(
            dropdown_option_index_from_mouse(rect, rect.y + 38.0 * 1.1, 14.95),
            Some(15),
            "top popup row should match the mostly visible option when smooth-scrolled"
        );
        assert_eq!(
            dropdown_option_index_from_mouse(rect, rect.y + 38.0 * 4.5, 20.3),
            Some(23),
            "deep popup rows should select the same option that rendering shows"
        );
    }

    #[test]
    fn dropdown_option_index_ignores_closed_control_row() {
        let rect = UiRect::new(100.0, 40.0, 250.0, 38.0);

        assert_eq!(
            dropdown_option_index_from_mouse(rect, rect.y + 12.0, 3.5),
            None
        );
    }
}
