use amigo_assets::AssetKey;
use amigo_math::ColorRgba;

use crate::*;
use super::style::{
    parse_optional_color_rgba_hex, required_ui_text, ui_event_binding_from_component,
    ui_style_from_component,
};

pub(super) fn ui_document_from_component(
    target: &SceneUiTargetComponentDocument,
    root: &SceneUiNodeComponentDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<SceneUiDocument> {
    Ok(SceneUiDocument {
        target: ui_target_from_component(target),
        root: ui_node_from_component(root, scene_id, entity_id, component_kind)?,
    })
}

pub(super) fn ui_target_from_component(target: &SceneUiTargetComponentDocument) -> SceneUiTarget {
    match target.kind {
        SceneUiTargetTypeComponentDocument::ScreenSpace => SceneUiTarget::ScreenSpace {
            layer: match target.layer {
                crate::SceneUiLayerComponentDocument::Background => SceneUiLayer::Background,
                crate::SceneUiLayerComponentDocument::Hud => SceneUiLayer::Hud,
                crate::SceneUiLayerComponentDocument::Menu => SceneUiLayer::Menu,
                crate::SceneUiLayerComponentDocument::Debug => SceneUiLayer::Debug,
            },
            viewport: target.viewport.map(|viewport| SceneUiViewport {
                width: viewport.width,
                height: viewport.height,
                scaling: match viewport.scaling {
                    crate::SceneUiViewportScalingComponentDocument::Expand => {
                        SceneUiViewportScaling::Expand
                    }
                    crate::SceneUiViewportScalingComponentDocument::Fixed => {
                        SceneUiViewportScaling::Fixed
                    }
                    crate::SceneUiViewportScalingComponentDocument::Fit => {
                        SceneUiViewportScaling::Fit
                    }
                },
            }),
        },
    }
}

pub(super) fn ui_node_from_component(
    node: &SceneUiNodeComponentDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<SceneUiNode> {
    let kind = match node.kind {
        SceneUiNodeTypeComponentDocument::Panel => SceneUiNodeKind::Panel,
        SceneUiNodeTypeComponentDocument::GroupBox => SceneUiNodeKind::GroupBox {
            label: node.text.clone().unwrap_or_default(),
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::Row => SceneUiNodeKind::Row,
        SceneUiNodeTypeComponentDocument::Column => SceneUiNodeKind::Column,
        SceneUiNodeTypeComponentDocument::Stack => SceneUiNodeKind::Stack,
        SceneUiNodeTypeComponentDocument::Text => SceneUiNodeKind::Text {
            content: required_ui_text(node, scene_id, entity_id, component_kind, "text")?,
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::Button => SceneUiNodeKind::Button {
            text: required_ui_text(node, scene_id, entity_id, component_kind, "button text")?,
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::ProgressBar => SceneUiNodeKind::ProgressBar {
            value: node.value.unwrap_or(0.0).clamp(0.0, 1.0),
        },
        SceneUiNodeTypeComponentDocument::Slider => SceneUiNodeKind::Slider {
            value: node.value.unwrap_or(0.0),
            min: node.min.unwrap_or(0.0),
            max: node.max.unwrap_or(1.0),
            step: node.step.unwrap_or(0.0).max(0.0),
        },
        SceneUiNodeTypeComponentDocument::Toggle => SceneUiNodeKind::Toggle {
            checked: node.checked.unwrap_or(false),
            text: node.text.clone().unwrap_or_default(),
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::OptionSet => SceneUiNodeKind::OptionSet {
            selected: node
                .selected
                .clone()
                .or_else(|| node.options.first().cloned())
                .unwrap_or_default(),
            options: node.options.clone(),
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::Dropdown => SceneUiNodeKind::Dropdown {
            selected: node
                .selected
                .clone()
                .or_else(|| node.options.first().cloned())
                .unwrap_or_default(),
            options: node.options.clone(),
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::TabView => SceneUiNodeKind::TabView {
            selected: node
                .selected
                .clone()
                .or_else(|| node.tabs.first().map(|tab| tab.id.clone()))
                .unwrap_or_default(),
            tabs: node
                .tabs
                .iter()
                .map(|tab| SceneUiTab {
                    id: tab.id.clone(),
                    label: tab.label.clone(),
                })
                .collect(),
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::ColorPickerRgb => SceneUiNodeKind::ColorPickerRgb {
            color: parse_optional_color_rgba_hex(
                node.color.as_deref(),
                scene_id,
                entity_id,
                component_kind,
                "color",
            )?
            .unwrap_or(ColorRgba::WHITE),
        },
        SceneUiNodeTypeComponentDocument::CurveEditor => SceneUiNodeKind::CurveEditor {
            points: ui_curve_points_from_component(node),
        },
        SceneUiNodeTypeComponentDocument::Spacer => SceneUiNodeKind::Spacer,
    };

    Ok(SceneUiNode {
        id: node.id.clone(),
        kind,
        style_class: node.style_class.clone(),
        style: ui_style_from_component(&node.style, scene_id, entity_id, component_kind)?,
        binds: SceneUiBinds {
            text: node.text_bind.clone(),
            visible: node.visible_bind.clone(),
            enabled: node.enabled_bind.clone(),
            value: node.value_bind.clone(),
        },
        on_click: node.on_click.as_ref().map(ui_event_binding_from_component),
        on_change: node.on_change.as_ref().map(ui_event_binding_from_component),
        children: node
            .children
            .iter()
            .map(|child| ui_node_from_component(child, scene_id, entity_id, component_kind))
            .collect::<SceneDocumentResult<Vec<_>>>()?,
    })
}

pub(super) fn ui_curve_points_from_component(node: &SceneUiNodeComponentDocument) -> Vec<SceneUiCurvePoint> {
    let points = if node.points.is_empty() {
        let denominator = node.values.len().saturating_sub(1).max(1) as f32;
        node.values
            .iter()
            .enumerate()
            .map(|(index, value)| SceneUiCurvePoint {
                t: index as f32 / denominator,
                value: *value,
            })
            .collect::<Vec<_>>()
    } else {
        node.points
            .iter()
            .map(|point| SceneUiCurvePoint {
                t: point.t,
                value: point.value,
            })
            .collect::<Vec<_>>()
    };

    normalize_scene_ui_curve_points(points)
}

pub(super) fn normalize_scene_ui_curve_points(mut points: Vec<SceneUiCurvePoint>) -> Vec<SceneUiCurvePoint> {
    if points.is_empty() {
        points = vec![
            SceneUiCurvePoint { t: 0.0, value: 0.0 },
            SceneUiCurvePoint {
                t: 1.0 / 3.0,
                value: 1.0 / 3.0,
            },
            SceneUiCurvePoint {
                t: 2.0 / 3.0,
                value: 2.0 / 3.0,
            },
            SceneUiCurvePoint { t: 1.0, value: 1.0 },
        ];
    }

    for point in &mut points {
        point.t = point.t.clamp(0.0, 1.0);
        point.value = point.value.clamp(0.0, 1.0);
    }
    points.sort_by(|a, b| a.t.total_cmp(&b.t));

    while points.len() < 4 {
        let t = (points.len() as f32 / 3.0).clamp(0.0, 1.0);
        points.push(SceneUiCurvePoint { t, value: t });
        points.sort_by(|a, b| a.t.total_cmp(&b.t));
    }

    points
}

