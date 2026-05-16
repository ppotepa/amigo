use std::collections::BTreeMap;

use amigo_core::{AmigoError, AmigoResult};
use amigo_math::ColorRgba;
use amigo_render_wgpu::{
    UiLayoutNode, UiOverlayCurvePoint, UiOverlayDocument, UiOverlayLayer, UiOverlayNode,
    UiOverlayNodeKind, UiOverlayStyle, UiOverlayTab, UiOverlayTextGlow, UiOverlayTextOutline,
    UiOverlayTextShadow, UiOverlayViewport, UiOverlayViewportScaling, UiRect, build_ui_layout_tree,
};
use amigo_runtime::Runtime;
use amigo_scripting_api::{ScriptEvent, ScriptEventQueue};

use crate::{
    UiCurvePoint, UiEventBinding, UiInputService, UiInputViewportState, UiSceneService,
    UiStateService, UiStateSnapshot, UiTextAlign, UiTheme, UiThemeService, UiViewportScaling,
    curve_editor_edit_from_mouse,
};

fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> AmigoResult<std::sync::Arc<T>> {
    runtime.resolve::<T>().ok_or_else(|| {
        AmigoError::Message(format!(
            "required service `{}` is not registered",
            std::any::type_name::<T>()
        ))
    })
}

pub struct ResolvedUiOverlayDocument {
    pub overlay: UiOverlayDocument,
    pub click_bindings: BTreeMap<String, UiEventBinding>,
    pub change_bindings: BTreeMap<String, UiEventBinding>,
}

pub trait UiOverlayRenderOutput {
    fn push_ui_overlay_document(&mut self, document: UiOverlayDocument);
}

impl UiOverlayRenderOutput for amigo_render_wgpu::WgpuRenderFramePacket {
    fn push_ui_overlay_document(&mut self, document: UiOverlayDocument) {
        self.push_game_ui_overlay(document);
    }
}

pub struct UiOverlayRenderExtractor;

impl UiOverlayRenderExtractor {
    pub fn name(&self) -> &'static str {
        "ui_overlay"
    }

    pub fn extract(
        &self,
        ui_scene_service: &UiSceneService,
        ui_state_service: &UiStateService,
        ui_theme_service: &UiThemeService,
        output: &mut impl UiOverlayRenderOutput,
    ) {
        for document in
            resolve_ui_overlay_documents(ui_scene_service, ui_state_service, ui_theme_service)
        {
            output.push_ui_overlay_document(document.overlay);
        }
    }
}

pub fn resolve_ui_overlay_documents(
    ui_scene_service: &UiSceneService,
    ui_state_service: &UiStateService,
    ui_theme_service: &UiThemeService,
) -> Vec<ResolvedUiOverlayDocument> {
    let snapshot = ui_state_service.snapshot();
    let active_theme = ui_theme_service.active_theme();
    let mut documents = ui_scene_service
        .commands()
        .into_iter()
        .filter_map(|command| {
            resolve_ui_overlay_document(
                &command.entity_name,
                &command.document,
                &snapshot,
                active_theme.as_ref(),
            )
        })
        .collect::<Vec<_>>();
    documents.sort_by_key(|document| document.overlay.layer);
    documents
}

pub fn process_ui_input(runtime: &Runtime) -> AmigoResult<()> {
    let viewport = required::<UiInputViewportState>(runtime)?.get();
    let Some(viewport) = viewport else {
        return Ok(());
    };

    let ui_input = required::<UiInputService>(runtime)?;
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

    let ui_scene = required::<UiSceneService>(runtime)?;
    let ui_state = required::<UiStateService>(runtime)?;
    let ui_theme = required::<UiThemeService>(runtime)?;
    let script_event_queue = required::<ScriptEventQueue>(runtime)?;
    let resolved =
        resolve_ui_overlay_documents(ui_scene.as_ref(), ui_state.as_ref(), ui_theme.as_ref());
    if snapshot.mouse_left_released {
        if let Some(active_path) = ui_input.active_path() {
            ui_state.clear_background(&active_path);
            ui_input.set_active_path(None);
        }
    }
    for document in resolved.iter().rev() {
        let layout = build_ui_layout_tree(viewport, &document.overlay);
        let Some(path) = hit_test_ui_layout(&layout, mouse_position.x, mouse_position.y) else {
            continue;
        };
        if !ui_state.is_enabled(&path) {
            continue;
        }
        let Some(layout_node) = find_ui_layout_node(&layout, &path) else {
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
                let edit_rect = crate::UiRect::new(
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
            let effective_scroll_offset = *scroll_offset;
            if snapshot.mouse_wheel_y.abs() > f32::EPSILON && *expanded {
                let visible_count = dropdown_visible_option_count(options.len());
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
                let visible_count = dropdown_visible_option_count(options.len());
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
                let visible_count = dropdown_visible_option_count(options.len());
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

pub fn hit_test_ui_layout(node: &UiLayoutNode, x: f32, y: f32) -> Option<String> {
    if let Some(path) = hit_test_expanded_dropdown(node, x, y) {
        return Some(path);
    }
    hit_test_ui_layout_normal(node, x, y)
}

pub fn dropdown_visible_option_count(option_count: usize) -> usize {
    option_count.min(10)
}

pub fn find_ui_layout_node<'a>(node: &'a UiLayoutNode, path: &str) -> Option<&'a UiLayoutNode> {
    if node.path == path {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_ui_layout_node(child, path) {
            return Some(found);
        }
    }
    None
}

fn resolve_ui_overlay_document(
    entity_name: &str,
    document: &crate::UiDocument,
    snapshot: &UiStateSnapshot,
    active_theme: Option<&UiTheme>,
) -> Option<ResolvedUiOverlayDocument> {
    let root_segment = document
        .root
        .id
        .clone()
        .unwrap_or_else(|| "root".to_owned());
    let root_path = format!("{entity_name}.{root_segment}");
    let mut click_bindings = BTreeMap::new();
    let mut change_bindings = BTreeMap::new();
    let root = resolve_ui_overlay_node(
        &document.root,
        &root_path,
        snapshot,
        active_theme,
        &mut click_bindings,
        &mut change_bindings,
    )?;
    Some(ResolvedUiOverlayDocument {
        overlay: UiOverlayDocument {
            entity_name: entity_name.to_owned(),
            layer: match document.target.layer() {
                crate::UiLayer::Background => UiOverlayLayer::Background,
                crate::UiLayer::Hud => UiOverlayLayer::Hud,
                crate::UiLayer::Menu => UiOverlayLayer::Menu,
                crate::UiLayer::Debug => UiOverlayLayer::Debug,
            },
            viewport: match document.target {
                crate::UiTarget::ScreenSpace { viewport, .. } => {
                    viewport.map(|viewport| UiOverlayViewport {
                        width: viewport.width,
                        height: viewport.height,
                        scaling: match viewport.scaling {
                            UiViewportScaling::Expand => UiOverlayViewportScaling::Expand,
                            UiViewportScaling::Fixed => UiOverlayViewportScaling::Fixed,
                            UiViewportScaling::Fit => UiOverlayViewportScaling::Fit,
                        },
                    })
                }
            },
            root,
        },
        click_bindings,
        change_bindings,
    })
}

fn resolve_ui_overlay_node(
    node: &crate::UiNode,
    path: &str,
    snapshot: &UiStateSnapshot,
    active_theme: Option<&UiTheme>,
    click_bindings: &mut BTreeMap<String, UiEventBinding>,
    change_bindings: &mut BTreeMap<String, UiEventBinding>,
) -> Option<UiOverlayNode> {
    if snapshot
        .visibility_overrides
        .get(path)
        .copied()
        .unwrap_or(true)
        == false
    {
        return None;
    }
    let kind = match &node.kind {
        crate::UiNodeKind::Panel => UiOverlayNodeKind::Panel,
        crate::UiNodeKind::GroupBox { label, font } => UiOverlayNodeKind::GroupBox {
            label: snapshot
                .text_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| label.clone()),
            font: font.clone(),
        },
        crate::UiNodeKind::Row => UiOverlayNodeKind::Row,
        crate::UiNodeKind::Column => UiOverlayNodeKind::Column,
        crate::UiNodeKind::Stack => UiOverlayNodeKind::Stack,
        crate::UiNodeKind::Text { content, font } => UiOverlayNodeKind::Text {
            content: snapshot
                .text_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| content.clone()),
            font: font.clone(),
        },
        crate::UiNodeKind::Button { text, font } => UiOverlayNodeKind::Button {
            text: snapshot
                .text_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| text.clone()),
            font: font.clone(),
        },
        crate::UiNodeKind::ProgressBar { value } => UiOverlayNodeKind::ProgressBar {
            value: snapshot
                .value_overrides
                .get(path)
                .copied()
                .unwrap_or(*value),
        },
        crate::UiNodeKind::Slider {
            value,
            min,
            max,
            step,
        } => UiOverlayNodeKind::Slider {
            value: snapshot
                .value_overrides
                .get(path)
                .copied()
                .unwrap_or(*value),
            min: *min,
            max: *max,
            step: *step,
        },
        crate::UiNodeKind::Toggle {
            checked,
            text,
            font,
        } => UiOverlayNodeKind::Toggle {
            checked: snapshot
                .value_overrides
                .get(path)
                .map(|value| *value >= 0.5)
                .unwrap_or(*checked),
            text: text.clone(),
            font: font.clone(),
        },
        crate::UiNodeKind::OptionSet {
            selected,
            options,
            font,
        } => UiOverlayNodeKind::OptionSet {
            selected: snapshot
                .selected_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| selected.clone()),
            options: snapshot
                .options_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| options.clone()),
            font: font.clone(),
        },
        crate::UiNodeKind::Dropdown {
            selected,
            options,
            font,
        } => UiOverlayNodeKind::Dropdown {
            selected: snapshot
                .selected_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| selected.clone()),
            options: snapshot
                .options_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| options.clone()),
            expanded: snapshot
                .expanded_overrides
                .get(path)
                .copied()
                .unwrap_or(false),
            scroll_offset: snapshot
                .dropdown_scroll_offsets
                .get(path)
                .copied()
                .unwrap_or(0.0),
            font: font.clone(),
        },
        crate::UiNodeKind::TabView {
            selected,
            tabs,
            font,
        } => UiOverlayNodeKind::TabView {
            selected: snapshot
                .selected_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| selected.clone()),
            tabs: tabs
                .iter()
                .map(|tab| UiOverlayTab {
                    id: tab.id.clone(),
                    label: tab.label.clone(),
                })
                .collect(),
            font: font.clone(),
        },
        crate::UiNodeKind::ColorPickerRgb { color } => UiOverlayNodeKind::ColorPickerRgb {
            color: snapshot
                .color_overrides
                .get(path)
                .copied()
                .unwrap_or(*color),
        },
        crate::UiNodeKind::CurveEditor { points } => UiOverlayNodeKind::CurveEditor {
            points: snapshot
                .curve_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| points.clone())
                .into_iter()
                .map(|point| UiOverlayCurvePoint {
                    t: point.t,
                    value: point.value,
                })
                .collect(),
        },
        crate::UiNodeKind::Spacer => UiOverlayNodeKind::Spacer,
    };

    let mut children = Vec::new();
    for (index, child) in node.children.iter().enumerate() {
        let segment = child
            .id
            .clone()
            .unwrap_or_else(|| format!("{}-{index}", child.kind.label()));
        let child_path = format!("{path}.{segment}");
        if let Some(child) = resolve_ui_overlay_node(
            child,
            &child_path,
            snapshot,
            active_theme,
            click_bindings,
            change_bindings,
        ) {
            children.push(child);
        }
    }
    if let Some(binding) = node.events.on_click.as_ref() {
        click_bindings.insert(path.to_owned(), binding.clone());
    }
    if let Some(binding) = node.events.on_change.as_ref() {
        change_bindings.insert(path.to_owned(), binding.clone());
    }

    Some(UiOverlayNode {
        id: node.id.clone(),
        kind,
        style: resolve_style(
            active_theme,
            node.style_class.as_deref(),
            &node.style,
            path,
            snapshot,
        ),
        children,
    })
}

fn resolve_style(
    active_theme: Option<&UiTheme>,
    style_class: Option<&str>,
    style: &crate::UiStyle,
    path: &str,
    snapshot: &UiStateSnapshot,
) -> UiOverlayStyle {
    let mut merged = active_theme
        .and_then(|theme| style_class.and_then(|style_class| theme.classes.get(style_class)))
        .cloned()
        .unwrap_or_default();
    merged.left = style.left.or(merged.left);
    merged.top = style.top.or(merged.top);
    merged.right = style.right.or(merged.right);
    merged.bottom = style.bottom.or(merged.bottom);
    merged.width = style.width.or(merged.width);
    merged.height = style.height.or(merged.height);
    merged.background = style.background.or(merged.background);
    merged.color = style.color.or(merged.color);
    merged.border_color = style.border_color.or(merged.border_color);
    merged.opacity = style.opacity.or(merged.opacity);
    merged.blend = style.blend.or(merged.blend);
    merged.text_shadow = style.text_shadow.or(merged.text_shadow);
    merged.text_outline = style.text_outline.or(merged.text_outline);
    merged.text_glow = style.text_glow.or(merged.text_glow);

    let default_style = crate::UiStyle::default();
    if style.padding != default_style.padding {
        merged.padding = style.padding;
    }
    if style.gap != default_style.gap {
        merged.gap = style.gap;
    }
    if style.border_width != default_style.border_width {
        merged.border_width = style.border_width;
    }
    if style.border_radius != default_style.border_radius {
        merged.border_radius = style.border_radius;
    }
    if style.font_size != default_style.font_size {
        merged.font_size = style.font_size;
    }
    if style.word_wrap != default_style.word_wrap {
        merged.word_wrap = style.word_wrap;
    }
    if style.fit_to_width != default_style.fit_to_width {
        merged.fit_to_width = style.fit_to_width;
    }
    if style.align != default_style.align {
        merged.align = style.align;
    }

    if let Some(height) = snapshot.height_overrides.get(path).copied() {
        merged.height = Some(height);
    }
    let opacity = merged.opacity.unwrap_or(1.0).clamp(0.0, 1.0);
    let mut overlay = UiOverlayStyle {
        left: merged.left,
        top: merged.top,
        right: merged.right,
        bottom: merged.bottom,
        width: merged.width,
        height: merged.height,
        padding: merged.padding,
        gap: merged.gap,
        background: merged
            .background
            .map(|color| color_with_alpha_mul(color, opacity)),
        color: merged
            .color
            .map(|color| color_with_alpha_mul(color, opacity)),
        border_color: merged
            .border_color
            .map(|color| color_with_alpha_mul(color, opacity)),
        opacity,
        border_width: merged.border_width,
        border_radius: merged.border_radius,
        font_size: merged.font_size,
        word_wrap: merged.word_wrap,
        fit_to_width: merged.fit_to_width,
        text_anchor: match merged.align {
            UiTextAlign::Start => amigo_render_wgpu::UiTextAnchor::TopLeft,
            UiTextAlign::Center => amigo_render_wgpu::UiTextAnchor::Center,
        },
        text_shadow: merged.text_shadow.map(|shadow| UiOverlayTextShadow {
            color: color_with_alpha_mul(shadow.color, opacity),
            offset: shadow.offset,
        }),
        text_outline: merged.text_outline.map(|outline| UiOverlayTextOutline {
            color: color_with_alpha_mul(outline.color, opacity),
            width: outline.width,
        }),
        text_glow: merged.text_glow.map(|glow| UiOverlayTextGlow {
            color: color_with_alpha_mul(glow.color, opacity),
            radius: glow.radius,
            intensity: glow.intensity,
            passes: glow.passes,
        }),
    };
    if let Some(color) = snapshot.color_overrides.get(path).copied() {
        overlay.color = Some(color);
    }
    if let Some(background) = snapshot.background_overrides.get(path).copied() {
        overlay.background = Some(background);
    }
    overlay
}

fn color_with_alpha_mul(color: amigo_math::ColorRgba, opacity: f32) -> amigo_math::ColorRgba {
    amigo_math::ColorRgba::new(color.r, color.g, color.b, color.a * opacity.clamp(0.0, 1.0))
}

fn hit_test_ui_layout_normal(node: &UiLayoutNode, x: f32, y: f32) -> Option<String> {
    if x < node.rect.x
        || y < node.rect.y
        || x > node.rect.x + node.rect.width
        || y > node.rect.y + node.rect.height
    {
        return None;
    }
    for child in node.children.iter().rev() {
        if let Some(path) = hit_test_ui_layout_normal(child, x, y) {
            return Some(path);
        }
    }
    Some(node.path.clone())
}

fn hit_test_expanded_dropdown(node: &UiLayoutNode, x: f32, y: f32) -> Option<String> {
    for child in node.children.iter().rev() {
        if let Some(path) = hit_test_expanded_dropdown(child, x, y) {
            return Some(path);
        }
    }
    let UiOverlayNodeKind::Dropdown {
        expanded: true,
        options,
        ..
    } = &node.node.kind
    else {
        return None;
    };
    let row_height = 38.0_f32.min(node.rect.height.max(0.0));
    let total_height = row_height * (dropdown_visible_option_count(options.len()) as f32 + 1.0);
    if x >= node.rect.x
        && x <= node.rect.x + node.rect.width
        && y >= node.rect.y
        && y <= node.rect.y + total_height
    {
        return Some(node.path.clone());
    }
    None
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

fn pressed_background(node: &UiLayoutNode) -> ColorRgba {
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
    let visible_count = dropdown_visible_option_count(option_count);
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
