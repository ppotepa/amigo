use amigo_math::ColorRgba;
use amigo_overlay_api::{
    UiOverlayDocument, UiOverlayLayer, UiOverlayNode, UiOverlayNodeKind, UiOverlayPreviewTriangle,
    UiOverlayStyle, UiOverlayTab,
};
use amigo_panel_api::{PanelSnapshot, PropertySnapshot};
use amigo_runtime_control::ControlValue;
use amigo_scene::{SceneUiNodeComponentDocument as Node, SceneUiNodeTypeComponentDocument as Kind};
use std::collections::BTreeMap;

const PANEL_WIDTH: f32 = 392.0;
const CARD_GAP: f32 = 8.0;
const CARD_WIDTH: f32 = 172.0;

pub fn overlay(
    snapshot: &PanelSnapshot,
    tabs: &BTreeMap<String, String>,
    collapsed: &BTreeMap<String, bool>,
    hovered: &BTreeMap<String, String>,
    dropdowns: &BTreeMap<String, bool>,
    dropdown_scrolls: &BTreeMap<String, f32>,
    scroll_offsets: &BTreeMap<String, f32>,
) -> UiOverlayDocument {
    UiOverlayDocument {
        entity_name: format!("panel:{}", snapshot.document.id),
        layer: UiOverlayLayer::Menu,
        viewport: None,
        root: UiOverlayNode {
            id: Some(format!("panel:{}", snapshot.document.id)),
            kind: UiOverlayNodeKind::Panel,
            style: UiOverlayStyle {
                right: Some(20.0),
                top: Some(20.0),
                bottom: Some(20.0),
                width: Some(PANEL_WIDTH),
                padding: 14.0,
                gap: 10.0,
                background: Some(ColorRgba::new(0.035, 0.047, 0.067, 0.96)),
                border_color: Some(ColorRgba::new(0.22, 0.31, 0.39, 1.0)),
                border_width: 1.0,
                border_radius: 10.0,
                ..UiOverlayStyle::default()
            },
            children: vec![
                UiOverlayNode {
                    id: Some("title".into()),
                    kind: UiOverlayNodeKind::Text {
                        content: snapshot.document.title.clone(),
                        font: None,
                    },
                    style: UiOverlayStyle {
                        font_size: 20.0,
                        color: Some(ColorRgba::new(0.86, 0.92, 0.97, 1.0)),
                        ..UiOverlayStyle::default()
                    },
                    children: vec![],
                },
                UiOverlayNode {
                    id: Some(format!("panel-scroll/{}", snapshot.document.id)),
                    kind: UiOverlayNodeKind::ScrollArea {
                        offset_y: scroll_offsets
                            .get(&snapshot.document.id)
                            .copied()
                            .unwrap_or_default(),
                    },
                    style: UiOverlayStyle {
                        fill_height: true,
                        ..UiOverlayStyle::default()
                    },
                    children: vec![render_node(
                        &snapshot.document.root,
                        snapshot,
                        tabs,
                        collapsed,
                        hovered,
                        dropdowns,
                        dropdown_scrolls,
                    )],
                },
            ],
        },
    }
}

pub fn node_by_id<'a>(snapshot: &'a PanelSnapshot, id: &str) -> Option<&'a Node> {
    snapshot
        .document
        .nodes()
        .into_iter()
        .find(|node| node.id.as_deref() == Some(id))
}

/// Stable identity for one direct-selection card in an authored option set.
///
/// The value is never parsed back out of the identifier. `choice_value_for_id`
/// resolves it against the current document, so authored values may contain
/// whitespace or punctuation without becoming an input protocol.
pub fn choice_id(panel_id: &str, control_id: &str, value: &str) -> String {
    format!("panel-choice/{panel_id}/{control_id}/{value}")
}

/// Resolves a hit-tested card to the option-set control and exact authored
/// value it represents.
pub fn choice_value_for_id(snapshot: &PanelSnapshot, id: &str) -> Option<(String, String)> {
    snapshot.document.nodes().into_iter().find_map(|node| {
        (node.kind == Kind::OptionSet)
            .then_some(node)
            .and_then(|node| {
                let control_id = node.id.as_deref()?;
                resolved_options(node, snapshot)
                    .iter()
                    .find(|value| choice_id(&snapshot.document.id, control_id, value) == id)
                    .map(|value| (control_id.to_owned(), value.clone()))
            })
    })
}

fn resolved_options(node: &Node, snapshot: &PanelSnapshot) -> Vec<String> {
    node.options_bind
        .as_ref()
        .and_then(|path| snapshot.values.get(path))
        .and_then(|value| value.value.as_string())
        .map(|value| {
            value
                .split('\n')
                .filter(|option| !option.is_empty())
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| node.options.clone())
}

fn render_node(
    node: &Node,
    snapshot: &PanelSnapshot,
    tabs: &BTreeMap<String, String>,
    collapsed: &BTreeMap<String, bool>,
    hovered: &BTreeMap<String, String>,
    dropdowns: &BTreeMap<String, bool>,
    dropdown_scrolls: &BTreeMap<String, f32>,
) -> UiOverlayNode {
    let id = node.id.clone();
    let value = node
        .value_bind
        .as_ref()
        .and_then(|path| snapshot.values.get(path))
        .map(|value| &value.value);
    let style = UiOverlayStyle {
        left: node.style.left,
        top: node.style.top,
        right: node.style.right,
        bottom: node.style.bottom,
        width: node.style.width,
        height: node.style.height,
        padding: node.style.padding,
        gap: node.style.gap,
        opacity: node.style.opacity.unwrap_or(1.0),
        border_width: node.style.border_width,
        border_radius: node.style.border_radius,
        font_size: node.style.font_size,
        word_wrap: node.style.word_wrap,
        fit_to_width: node.style.fit_to_width,
        ..UiOverlayStyle::default()
    };
    let uses_choice_cards = uses_choice_cards(node, snapshot);
    let group_collapsed = group_is_collapsed(snapshot, node, collapsed);
    let children = if group_collapsed {
        Vec::new()
    } else if uses_choice_cards {
        option_cards(node, snapshot, value, hovered)
    } else {
        node.children
            .iter()
            .filter(|child| visible(child, snapshot))
            .map(|child| {
                render_node(
                    child,
                    snapshot,
                    tabs,
                    collapsed,
                    hovered,
                    dropdowns,
                    dropdown_scrolls,
                )
            })
            .collect()
    };
    UiOverlayNode {
        id,
        kind: match node.kind {
            Kind::Panel => UiOverlayNodeKind::Panel,
            Kind::GroupBox => UiOverlayNodeKind::GroupBox {
                label: node.text.clone().unwrap_or_default(),
                font: None,
            },
            Kind::Row => UiOverlayNodeKind::Row,
            Kind::Column => UiOverlayNodeKind::Column,
            Kind::Stack => UiOverlayNodeKind::Stack,
            Kind::Text => UiOverlayNodeKind::Text {
                content: text(node, value),
                font: None,
            },
            Kind::Button => UiOverlayNodeKind::Button {
                text: node.text.clone().unwrap_or_default(),
                font: None,
            },
            Kind::ProgressBar => UiOverlayNodeKind::ProgressBar {
                value: number(value).unwrap_or(node.value.unwrap_or_default()),
            },
            Kind::Slider => UiOverlayNodeKind::Slider {
                value: number(value).unwrap_or(node.value.unwrap_or_default()),
                min: node.min.unwrap_or(0.0),
                max: node.max.unwrap_or(1.0),
                step: node.step.unwrap_or(0.01),
            },
            Kind::Toggle => UiOverlayNodeKind::Toggle {
                checked: value
                    .and_then(ControlValue::as_bool)
                    .unwrap_or(node.checked.unwrap_or(false)),
                text: node.text.clone().unwrap_or_default(),
                font: None,
            },
            // An option set is a gallery of direct-selection cards. Keeping
            // the interaction as ordinary buttons means it uses the existing
            // overlay transport while preserving the authored option list as
            // the single source of truth.
            Kind::OptionSet if uses_choice_cards => UiOverlayNodeKind::Column,
            Kind::OptionSet => UiOverlayNodeKind::OptionSet {
                selected: value
                    .and_then(ControlValue::as_string)
                    .unwrap_or_default()
                    .to_owned(),
                options: resolved_options(node, snapshot),
                font: None,
            },
            Kind::Dropdown => UiOverlayNodeKind::Dropdown {
                selected: value
                    .and_then(ControlValue::as_string)
                    .unwrap_or_default()
                    .to_owned(),
                options: resolved_options(node, snapshot),
                expanded: node
                    .id
                    .as_ref()
                    .and_then(|id| dropdowns.get(&format!("{}:{id}", snapshot.document.id)))
                    .copied()
                    .unwrap_or(false),
                scroll_offset: node
                    .id
                    .as_ref()
                    .and_then(|id| dropdown_scrolls.get(&format!("{}:{id}", snapshot.document.id)))
                    .copied()
                    .unwrap_or(0.0),
                font: None,
            },
            Kind::TabView => UiOverlayNodeKind::TabView {
                selected: node
                    .id
                    .as_ref()
                    .and_then(|id| tabs.get(&format!("{}:{id}", snapshot.document.id)))
                    .filter(|selected| node.tabs.iter().any(|tab| tab.id == **selected))
                    .cloned()
                    .or_else(|| node.tabs.first().map(|tab| tab.id.clone()))
                    .unwrap_or_default(),
                tabs: node
                    .tabs
                    .iter()
                    .map(|tab| UiOverlayTab {
                        id: tab.id.clone(),
                        label: tab.label.clone(),
                    })
                    .collect(),
                font: None,
            },
            Kind::ColorPickerRgb => UiOverlayNodeKind::ColorPickerRgb {
                color: ColorRgba::WHITE,
            },
            Kind::CurveEditor => UiOverlayNodeKind::CurveEditor { points: vec![] },
            Kind::Spacer => UiOverlayNodeKind::Spacer,
        },
        style,
        children,
    }
}

pub fn group_is_collapsed(
    snapshot: &PanelSnapshot,
    node: &Node,
    collapsed: &BTreeMap<String, bool>,
) -> bool {
    if node.kind != Kind::GroupBox {
        return false;
    }
    let Some(id) = node.id.as_deref() else {
        return false;
    };
    collapsed
        .get(&format!("{}:{id}", snapshot.document.id))
        .copied()
        .unwrap_or_else(|| {
            snapshot
                .document
                .presentation
                .get(id)
                .is_some_and(|presentation| presentation.collapsed)
        })
}

fn uses_choice_cards(node: &Node, snapshot: &PanelSnapshot) -> bool {
    node.kind == Kind::OptionSet
        && resolved_options(node, snapshot) == node.options
        && node
            .id
            .as_ref()
            .and_then(|id| snapshot.document.presentation.get(id))
            .is_some_and(|presentation| !presentation.choices.is_empty())
}

fn option_cards(
    node: &Node,
    snapshot: &PanelSnapshot,
    selected: Option<&ControlValue>,
    hovered: &BTreeMap<String, String>,
) -> Vec<UiOverlayNode> {
    let Some(control_id) = node.id.as_deref() else {
        return Vec::new();
    };
    let selected = selected
        .and_then(ControlValue::as_string)
        .unwrap_or_default();
    let presentation = snapshot.document.presentation.get(control_id);
    let cards = node
        .options
        .iter()
        .map(|value| {
            let choice = presentation
                .and_then(|entry| entry.choices.iter().find(|choice| choice.value == *value));
            let label = choice.map(|choice| choice.label.as_str()).unwrap_or(value);
            let status = choice
                .and_then(|choice| choice.status_bind.as_ref())
                .and_then(|path| snapshot.values.get(path))
                .map(|status| display(&status.value));
            let is_selected = selected == value;
            let card_id = choice_id(&snapshot.document.id, control_id, value);
            let is_hovered = hovered
                .get(&snapshot.document.id)
                .is_some_and(|id| id == &card_id);
            let preview_triangles = choice
                .and_then(|choice| artwork_key(choice, snapshot))
                .and_then(|key| snapshot.document.artwork.get(key))
                .map(|triangles| {
                    triangles
                        .iter()
                        .map(|triangle| UiOverlayPreviewTriangle {
                            points: triangle.points,
                            color: ColorRgba::new(
                                triangle.color[0] as f32 / 255.0,
                                triangle.color[1] as f32 / 255.0,
                                triangle.color[2] as f32 / 255.0,
                                0.72,
                            ),
                        })
                        .collect()
                })
                .unwrap_or_default();
            UiOverlayNode {
                id: Some(card_id),
                kind: UiOverlayNodeKind::Button {
                    text: status
                        .map(|status| format!("{}\n{}", label, status))
                        .unwrap_or_else(|| label.to_owned()),
                    font: None,
                },
                style: UiOverlayStyle {
                    width: Some(CARD_WIDTH),
                    height: Some(62.0),
                    padding: 9.0,
                    border_width: if is_selected { 2.0 } else { 1.0 },
                    border_radius: 7.0,
                    background: Some(if is_selected {
                        ColorRgba::new(0.12, 0.29, 0.40, 1.0)
                    } else if is_hovered {
                        ColorRgba::new(0.10, 0.16, 0.21, 1.0)
                    } else {
                        ColorRgba::new(0.065, 0.086, 0.115, 1.0)
                    }),
                    border_color: Some(if is_selected {
                        ColorRgba::new(0.32, 0.76, 0.91, 1.0)
                    } else if is_hovered {
                        ColorRgba::new(0.36, 0.56, 0.67, 1.0)
                    } else {
                        ColorRgba::new(0.19, 0.27, 0.34, 1.0)
                    }),
                    preview_triangles,
                    ..UiOverlayStyle::default()
                },
                children: vec![],
            }
        })
        .collect::<Vec<_>>();
    cards
        .chunks(2)
        .map(|pair| UiOverlayNode {
            id: None,
            kind: UiOverlayNodeKind::Row,
            style: UiOverlayStyle {
                height: Some(62.0),
                gap: CARD_GAP,
                ..UiOverlayStyle::default()
            },
            children: pair.to_vec(),
        })
        .collect()
}

fn artwork_key<'a>(
    choice: &'a amigo_panel_api::PanelChoice,
    snapshot: &'a PanelSnapshot,
) -> Option<&'a str> {
    choice
        .artwork_bind
        .as_ref()
        .and_then(|path| snapshot.values.get(path))
        .and_then(|value| value.value.as_string())
        .or(choice.artwork.as_deref())
}

fn visible(node: &Node, snapshot: &PanelSnapshot) -> bool {
    node.visible_bind
        .as_ref()
        .and_then(|path| snapshot.values.get(path))
        .and_then(|value| value.value.as_bool())
        .unwrap_or(true)
}

fn text(node: &Node, value: Option<&ControlValue>) -> String {
    match (node.text.as_deref(), value) {
        (_, Some(value)) if node.text_bind.is_some() => display(value),
        (Some(label), _) => label.to_owned(),
        (None, Some(value)) => display(value),
        (None, None) => String::new(),
    }
}

pub fn number(value: Option<&ControlValue>) -> Option<f32> {
    value
        .and_then(ControlValue::as_f64)
        .map(|number| number as f32)
}

pub fn editable_value<'a>(
    snapshot: &'a PanelSnapshot,
    node: &Node,
) -> Option<&'a PropertySnapshot> {
    node.value_bind
        .as_ref()
        .and_then(|path| snapshot.values.get(path))
        .filter(|value| value.writable)
}

fn display(value: &ControlValue) -> String {
    match value {
        ControlValue::Bool(value) => value.to_string(),
        ControlValue::I64(value) => value.to_string(),
        ControlValue::U64(value) => value.to_string(),
        ControlValue::F64(value) => format!("{value:.3}"),
        ControlValue::String(value) | ControlValue::AssetRef(value) => value.clone(),
        other => format!("{other:?}"),
    }
}
