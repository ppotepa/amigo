use amigo_assets::AssetKey;
use amigo_scene::{
    SceneUiBinds, SceneUiDocument, SceneUiEventBinding, SceneUiLayer, SceneUiNode, SceneUiNodeKind,
    SceneUiStyle, SceneUiTarget, SceneUiTextAlign,
};

use crate::{
    UiBinds, UiDocument, UiEventBinding, UiEvents, UiLayer, UiNode, UiNodeKind, UiStyle, UiTarget,
    UiTextAlign,
};

pub fn collect_scene_ui_font_asset_keys(document: &SceneUiDocument) -> Vec<AssetKey> {
    let mut fonts = Vec::new();
    collect_scene_ui_node_font_asset_keys(&document.root, &mut fonts);
    fonts
}

pub fn scene_ui_document_to_runtime_document(document: &SceneUiDocument) -> UiDocument {
    UiDocument {
        target: convert_scene_ui_target(&document.target),
        root: convert_scene_ui_node(&document.root),
    }
}

fn collect_scene_ui_node_font_asset_keys(node: &SceneUiNode, fonts: &mut Vec<AssetKey>) {
    match &node.kind {
        SceneUiNodeKind::Text { font, .. } | SceneUiNodeKind::Button { font, .. } => {
            if let Some(font) = font.as_ref() {
                fonts.push(font.clone());
            }
        }
        SceneUiNodeKind::Panel
        | SceneUiNodeKind::Row
        | SceneUiNodeKind::Column
        | SceneUiNodeKind::Stack
        | SceneUiNodeKind::ProgressBar { .. }
        | SceneUiNodeKind::Spacer => {}
    }

    for child in &node.children {
        collect_scene_ui_node_font_asset_keys(child, fonts);
    }
}

fn convert_scene_ui_target(target: &SceneUiTarget) -> UiTarget {
    match target {
        SceneUiTarget::ScreenSpace { layer } => UiTarget::ScreenSpace {
            layer: convert_scene_ui_layer(*layer),
        },
    }
}

fn convert_scene_ui_layer(layer: SceneUiLayer) -> UiLayer {
    match layer {
        SceneUiLayer::Background => UiLayer::Background,
        SceneUiLayer::Hud => UiLayer::Hud,
        SceneUiLayer::Menu => UiLayer::Menu,
        SceneUiLayer::Debug => UiLayer::Debug,
    }
}

fn convert_scene_ui_node(node: &SceneUiNode) -> UiNode {
    UiNode {
        id: node.id.clone(),
        kind: convert_scene_ui_node_kind(&node.kind),
        style: convert_scene_ui_style(&node.style),
        binds: convert_scene_ui_binds(&node.binds),
        events: UiEvents {
            on_click: node.on_click.as_ref().map(convert_scene_ui_event_binding),
        },
        children: node.children.iter().map(convert_scene_ui_node).collect(),
    }
}

fn convert_scene_ui_binds(binds: &SceneUiBinds) -> UiBinds {
    UiBinds {
        text: binds.text.clone(),
        visible: binds.visible.clone(),
        enabled: binds.enabled.clone(),
        value: binds.value.clone(),
    }
}

fn convert_scene_ui_node_kind(kind: &SceneUiNodeKind) -> UiNodeKind {
    match kind {
        SceneUiNodeKind::Panel => UiNodeKind::Panel,
        SceneUiNodeKind::Row => UiNodeKind::Row,
        SceneUiNodeKind::Column => UiNodeKind::Column,
        SceneUiNodeKind::Stack => UiNodeKind::Stack,
        SceneUiNodeKind::Text { content, font } => UiNodeKind::Text {
            content: content.clone(),
            font: font.clone(),
        },
        SceneUiNodeKind::Button { text, font } => UiNodeKind::Button {
            text: text.clone(),
            font: font.clone(),
        },
        SceneUiNodeKind::ProgressBar { value } => UiNodeKind::ProgressBar { value: *value },
        SceneUiNodeKind::Spacer => UiNodeKind::Spacer,
    }
}

fn convert_scene_ui_style(style: &SceneUiStyle) -> UiStyle {
    UiStyle {
        left: style.left,
        top: style.top,
        right: style.right,
        bottom: style.bottom,
        width: style.width,
        height: style.height,
        padding: style.padding,
        gap: style.gap,
        background: style.background,
        color: style.color,
        border_color: style.border_color,
        border_width: style.border_width,
        border_radius: style.border_radius,
        font_size: style.font_size,
        word_wrap: style.word_wrap,
        fit_to_width: style.fit_to_width,
        align: match style.align {
            SceneUiTextAlign::Start => UiTextAlign::Start,
            SceneUiTextAlign::Center => UiTextAlign::Center,
        },
    }
}

fn convert_scene_ui_event_binding(binding: &SceneUiEventBinding) -> UiEventBinding {
    UiEventBinding::new(binding.event.clone(), binding.payload.clone())
}

#[cfg(test)]
mod tests {
    use amigo_assets::AssetKey;
    use amigo_math::ColorRgba;
    use amigo_scene::{
        SceneUiDocument, SceneUiLayer, SceneUiNode, SceneUiNodeKind, SceneUiStyle, SceneUiTarget,
    };

    use super::{collect_scene_ui_font_asset_keys, scene_ui_document_to_runtime_document};

    #[test]
    fn collects_font_asset_keys_from_scene_ui_document() {
        let document = SceneUiDocument {
            target: SceneUiTarget::ScreenSpace {
                layer: SceneUiLayer::Hud,
            },
            root: SceneUiNode {
                id: Some("root".to_owned()),
                kind: SceneUiNodeKind::Column,
                style: SceneUiStyle::default(),
                binds: Default::default(),
                on_click: None,
                children: vec![
                    SceneUiNode {
                        id: Some("title".to_owned()),
                        kind: SceneUiNodeKind::Text {
                            content: "AMIGO UI".to_owned(),
                            font: Some(AssetKey::new("playground-2d/fonts/debug-ui")),
                        },
                        style: SceneUiStyle::default(),
                        binds: Default::default(),
                        on_click: None,
                        children: Vec::new(),
                    },
                    SceneUiNode {
                        id: Some("button".to_owned()),
                        kind: SceneUiNodeKind::Button {
                            text: "GO".to_owned(),
                            font: Some(AssetKey::new("playground-2d/fonts/debug-ui-bold")),
                        },
                        style: SceneUiStyle::default(),
                        binds: Default::default(),
                        on_click: None,
                        children: Vec::new(),
                    },
                ],
            },
        };

        let fonts = collect_scene_ui_font_asset_keys(&document);
        assert_eq!(
            fonts,
            vec![
                AssetKey::new("playground-2d/fonts/debug-ui"),
                AssetKey::new("playground-2d/fonts/debug-ui-bold"),
            ]
        );
    }

    #[test]
    fn converts_scene_ui_document_to_runtime_document() {
        let document = SceneUiDocument {
            target: SceneUiTarget::ScreenSpace {
                layer: SceneUiLayer::Hud,
            },
            root: SceneUiNode {
                id: Some("root".to_owned()),
                kind: SceneUiNodeKind::Column,
                style: SceneUiStyle {
                    width: Some(320.0),
                    background: Some(ColorRgba::new(0.0, 0.0, 0.0, 1.0)),
                    ..SceneUiStyle::default()
                },
                on_click: None,
                binds: Default::default(),
                children: vec![SceneUiNode {
                    id: Some("title".to_owned()),
                    kind: SceneUiNodeKind::Text {
                        content: "AMIGO UI".to_owned(),
                        font: Some(AssetKey::new("playground-2d/fonts/debug-ui")),
                    },
                    style: SceneUiStyle::default(),
                    binds: Default::default(),
                    on_click: None,
                    children: Vec::new(),
                }],
            },
        };

        let runtime = scene_ui_document_to_runtime_document(&document);
        assert_eq!(runtime.root.id.as_deref(), Some("root"));
        assert_eq!(runtime.root.children.len(), 1);
        assert_eq!(runtime.root.style.width, Some(320.0));
        assert_eq!(
            runtime.root.style.background,
            Some(ColorRgba::new(0.0, 0.0, 0.0, 1.0))
        );
    }
}
