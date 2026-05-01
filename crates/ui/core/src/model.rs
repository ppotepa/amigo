use std::collections::BTreeMap;

use amigo_assets::AssetKey;
use amigo_math::ColorRgba;

#[derive(Debug, Clone, PartialEq)]
pub struct UiDocument {
    pub target: UiTarget,
    pub root: UiNode,
}

impl UiDocument {
    pub fn screen_space(layer: UiLayer, root: UiNode) -> Self {
        Self {
            target: UiTarget::ScreenSpace {
                layer,
                viewport: None,
            },
            root,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiTarget {
    ScreenSpace {
        layer: UiLayer,
        viewport: Option<UiViewport>,
    },
}

impl UiTarget {
    pub fn layer(&self) -> UiLayer {
        match self {
            Self::ScreenSpace { layer, .. } => *layer,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiViewport {
    pub width: f32,
    pub height: f32,
    pub scaling: UiViewportScaling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiViewportScaling {
    Expand,
    Fixed,
    Fit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UiLayer {
    Background,
    Hud,
    Menu,
    Debug,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiNode {
    pub id: Option<String>,
    pub kind: UiNodeKind,
    pub style_class: Option<String>,
    pub style: UiStyle,
    pub binds: UiBinds,
    pub events: UiEvents,
    pub children: Vec<UiNode>,
}

impl UiNode {
    pub fn new(kind: UiNodeKind) -> Self {
        Self {
            id: None,
            kind,
            style_class: None,
            style: UiStyle::default(),
            binds: UiBinds::default(),
            events: UiEvents::default(),
            children: Vec::new(),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn with_style(mut self, style: UiStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_style_class(mut self, style_class: impl Into<String>) -> Self {
        self.style_class = Some(style_class.into());
        self
    }

    pub fn with_binds(mut self, binds: UiBinds) -> Self {
        self.binds = binds;
        self
    }

    pub fn with_children(mut self, children: Vec<UiNode>) -> Self {
        self.children = children;
        self
    }

    pub fn with_events(mut self, events: UiEvents) -> Self {
        self.events = events;
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiBinds {
    pub text: Option<String>,
    pub visible: Option<String>,
    pub enabled: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiNodeKind {
    Panel,
    GroupBox {
        label: String,
        font: Option<AssetKey>,
    },
    Row,
    Column,
    Stack,
    Text {
        content: String,
        font: Option<AssetKey>,
    },
    Button {
        text: String,
        font: Option<AssetKey>,
    },
    ProgressBar {
        value: f32,
    },
    Slider {
        value: f32,
        min: f32,
        max: f32,
        step: f32,
    },
    Toggle {
        checked: bool,
        text: String,
        font: Option<AssetKey>,
    },
    OptionSet {
        selected: String,
        options: Vec<String>,
        font: Option<AssetKey>,
    },
    Dropdown {
        selected: String,
        options: Vec<String>,
        font: Option<AssetKey>,
    },
    TabView {
        selected: String,
        tabs: Vec<UiTab>,
        font: Option<AssetKey>,
    },
    ColorPickerRgb {
        color: ColorRgba,
    },
    CurveEditor {
        points: Vec<UiCurvePoint>,
    },
    Spacer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiCurvePoint {
    pub t: f32,
    pub value: f32,
}

impl UiCurvePoint {
    pub const fn new(t: f32, value: f32) -> Self {
        Self { t, value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiCurveEdit {
    pub point_index: usize,
    pub point: UiCurvePoint,
    pub points: Vec<UiCurvePoint>,
}

impl UiCurveEdit {
    pub fn payload(&self) -> Vec<String> {
        let mut payload = vec![
            self.point_index.to_string(),
            format!("{:.4}", self.point.t),
            format!("{:.4}", self.point.value),
            format_curve_points(&self.points),
        ];
        for point in normalize_curve_points(&self.points).iter().take(4) {
            payload.push(format!("{:.4}", point.value));
        }
        payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiTab {
    pub id: String,
    pub label: String,
}

impl UiNodeKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Panel => "panel",
            Self::GroupBox { .. } => "group-box",
            Self::Row => "row",
            Self::Column => "column",
            Self::Stack => "stack",
            Self::Text { .. } => "text",
            Self::Button { .. } => "button",
            Self::ProgressBar { .. } => "progress-bar",
            Self::Slider { .. } => "slider",
            Self::Toggle { .. } => "toggle",
            Self::OptionSet { .. } => "option-set",
            Self::Dropdown { .. } => "dropdown",
            Self::TabView { .. } => "tab-view",
            Self::ColorPickerRgb { .. } => "color-picker-rgb",
            Self::CurveEditor { .. } => "curve-editor",
            Self::Spacer => "spacer",
        }
    }
}

pub fn normalize_curve_points(points: &[UiCurvePoint]) -> Vec<UiCurvePoint> {
    let mut normalized = if points.is_empty() {
        default_curve_points()
    } else {
        points
            .iter()
            .map(|point| UiCurvePoint {
                t: point.t.clamp(0.0, 1.0),
                value: point.value.clamp(0.0, 1.0),
            })
            .collect::<Vec<_>>()
    };
    normalized.sort_by(|a, b| a.t.total_cmp(&b.t));

    while normalized.len() < 4 {
        let t = (normalized.len() as f32 / 3.0).clamp(0.0, 1.0);
        normalized.push(UiCurvePoint::new(t, t));
        normalized.sort_by(|a, b| a.t.total_cmp(&b.t));
    }

    normalized
}

pub fn default_curve_points() -> Vec<UiCurvePoint> {
    vec![
        UiCurvePoint::new(0.0, 0.0),
        UiCurvePoint::new(1.0 / 3.0, 1.0 / 3.0),
        UiCurvePoint::new(2.0 / 3.0, 2.0 / 3.0),
        UiCurvePoint::new(1.0, 1.0),
    ]
}

pub fn curve_points_from_values(values: &[f32]) -> Vec<UiCurvePoint> {
    if values.is_empty() {
        return default_curve_points();
    }
    let denominator = (values.len().saturating_sub(1)).max(1) as f32;
    normalize_curve_points(
        &values
            .iter()
            .enumerate()
            .map(|(index, value)| UiCurvePoint::new(index as f32 / denominator, *value))
            .collect::<Vec<_>>(),
    )
}

pub fn format_curve_points(points: &[UiCurvePoint]) -> String {
    normalize_curve_points(points)
        .iter()
        .map(|point| format!("{:.4}:{:.4}", point.t, point.value))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn curve_editor_edit_from_mouse(
    rect: UiRect,
    points: &[UiCurvePoint],
    mouse_x: f32,
    mouse_y: f32,
) -> Option<UiCurveEdit> {
    if rect.width <= f32::EPSILON || rect.height <= f32::EPSILON {
        return None;
    }

    let mut points = normalize_curve_points(points);
    let t = ((mouse_x - rect.x) / rect.width).clamp(0.0, 1.0);
    let value = (1.0 - ((mouse_y - rect.y) / rect.height)).clamp(0.0, 1.0);
    let point_index = nearest_curve_point_index(&points, t);
    points[point_index] = UiCurvePoint::new(t, value);
    points.sort_by(|a, b| a.t.total_cmp(&b.t));
    let point_index = nearest_curve_point_index(&points, t);
    let point = points[point_index];

    Some(UiCurveEdit {
        point_index,
        point,
        points,
    })
}

fn nearest_curve_point_index(points: &[UiCurvePoint], t: f32) -> usize {
    points
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| (a.t - t).abs().total_cmp(&(b.t - t).abs()))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiStyle {
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub padding: f32,
    pub gap: f32,
    pub background: Option<ColorRgba>,
    pub color: Option<ColorRgba>,
    pub border_color: Option<ColorRgba>,
    pub border_width: f32,
    pub border_radius: f32,
    pub font_size: f32,
    pub word_wrap: bool,
    pub fit_to_width: bool,
    pub align: UiTextAlign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTextAlign {
    Start,
    Center,
}

impl Default for UiStyle {
    fn default() -> Self {
        Self {
            left: None,
            top: None,
            right: None,
            bottom: None,
            width: None,
            height: None,
            padding: 0.0,
            gap: 0.0,
            background: None,
            color: None,
            border_color: None,
            border_width: 0.0,
            border_radius: 0.0,
            font_size: 16.0,
            word_wrap: false,
            fit_to_width: false,
            align: UiTextAlign::Start,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiEvents {
    pub on_click: Option<UiEventBinding>,
    pub on_change: Option<UiEventBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiEventBinding {
    pub event: String,
    pub payload: Vec<String>,
}

impl UiEventBinding {
    pub fn new(event: impl Into<String>, payload: Vec<String>) -> Self {
        Self {
            event: event.into(),
            payload,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl UiRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x <= self.x + self.width && y <= self.y + self.height
    }

    pub fn inset(&self, amount: f32) -> Self {
        let double = amount * 2.0;
        Self {
            x: self.x + amount,
            y: self.y + amount,
            width: (self.width - double).max(0.0),
            height: (self.height - double).max(0.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiLayoutNode {
    pub path: String,
    pub rect: UiRect,
    pub node: UiNode,
    pub children: Vec<UiLayoutNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiTheme {
    pub id: String,
    pub palette: UiThemePalette,
    pub classes: BTreeMap<String, UiStyle>,
}

impl UiTheme {
    pub fn from_palette(id: impl Into<String>, palette: UiThemePalette) -> Self {
        Self {
            id: id.into(),
            classes: default_theme_classes(&palette),
            palette,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiThemePalette {
    pub background: ColorRgba,
    pub surface: ColorRgba,
    pub surface_alt: ColorRgba,
    pub text: ColorRgba,
    pub text_muted: ColorRgba,
    pub border: ColorRgba,
    pub accent: ColorRgba,
    pub accent_text: ColorRgba,
    pub danger: ColorRgba,
    pub warning: ColorRgba,
    pub success: ColorRgba,
}

fn default_theme_classes(palette: &UiThemePalette) -> BTreeMap<String, UiStyle> {
    let mut classes = BTreeMap::new();
    classes.insert(
        "root".to_owned(),
        UiStyle {
            background: Some(palette.background),
            color: Some(palette.text),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "top_bar".to_owned(),
        UiStyle {
            background: Some(palette.surface),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            padding: 16.0,
            gap: 12.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "bottom_bar".to_owned(),
        UiStyle {
            background: Some(palette.surface),
            color: Some(palette.text_muted),
            border_color: Some(palette.border),
            border_width: 1.0,
            padding: 10.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "tab_bar".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 10.0,
            padding: 6.0,
            gap: 6.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "tab".to_owned(),
        button_style(palette.surface_alt, palette.text_muted, palette.border),
    );
    classes.insert(
        "tab_active".to_owned(),
        button_style(palette.accent, palette.accent_text, palette.accent),
    );
    classes.insert(
        "tab_disabled".to_owned(),
        button_style(palette.surface_alt, palette.text_muted, palette.border),
    );
    classes.insert(
        "panel".to_owned(),
        UiStyle {
            background: Some(palette.surface),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 10.0,
            padding: 16.0,
            gap: 10.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "group_box".to_owned(),
        UiStyle {
            background: Some(palette.surface),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 10.0,
            padding: 14.0,
            gap: 8.0,
            font_size: 14.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "inspector".to_owned(),
        UiStyle {
            background: Some(palette.surface),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 10.0,
            padding: 14.0,
            gap: 8.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "inspector_section".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 8.0,
            padding: 10.0,
            gap: 6.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "inspector_row".to_owned(),
        UiStyle {
            color: Some(palette.text),
            gap: 8.0,
            height: Some(28.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "inspector_label".to_owned(),
        UiStyle {
            color: Some(palette.text_muted),
            font_size: 14.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "inspector_value".to_owned(),
        UiStyle {
            color: Some(palette.text),
            font_size: 14.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "panel_alt".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 10.0,
            padding: 14.0,
            gap: 8.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "card".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 8.0,
            padding: 12.0,
            gap: 6.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "card_selected".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.text),
            border_color: Some(palette.accent),
            border_width: 2.0,
            border_radius: 8.0,
            padding: 12.0,
            gap: 6.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "text_title".to_owned(),
        UiStyle {
            color: Some(palette.text),
            font_size: 28.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "text_body".to_owned(),
        UiStyle {
            color: Some(palette.text),
            font_size: 16.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "text_muted".to_owned(),
        UiStyle {
            color: Some(palette.text_muted),
            font_size: 14.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "button".to_owned(),
        button_style(palette.surface_alt, palette.text, palette.border),
    );
    classes.insert(
        "button_primary".to_owned(),
        button_style(palette.accent, palette.accent_text, palette.accent),
    );
    classes.insert(
        "button_secondary".to_owned(),
        button_style(palette.surface_alt, palette.text, palette.border),
    );
    classes.insert(
        "button_danger".to_owned(),
        button_style(palette.danger, palette.accent_text, palette.danger),
    );
    classes.insert(
        "button_selected".to_owned(),
        button_style(palette.accent, palette.accent_text, palette.accent),
    );
    classes.insert(
        "button_disabled".to_owned(),
        button_style(palette.surface_alt, palette.text_muted, palette.border),
    );
    classes.insert(
        "progress".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.accent),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 999.0,
            height: Some(18.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "slider".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.accent),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 999.0,
            height: Some(24.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "slider_track".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 999.0,
            height: Some(8.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "slider_fill".to_owned(),
        UiStyle {
            background: Some(palette.accent),
            border_radius: 999.0,
            height: Some(8.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "slider_thumb".to_owned(),
        UiStyle {
            background: Some(palette.accent),
            border_color: Some(palette.accent_text),
            border_width: 1.0,
            border_radius: 999.0,
            width: Some(14.0),
            height: Some(22.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "toggle".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 999.0,
            padding: 8.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "toggle_on".to_owned(),
        button_style(palette.success, palette.accent_text, palette.success),
    );
    classes.insert(
        "toggle_off".to_owned(),
        button_style(palette.surface_alt, palette.text_muted, palette.border),
    );
    classes.insert(
        "dropdown".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 8.0,
            height: Some(38.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "option_set".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.accent),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 8.0,
            height: Some(38.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "option_selected".to_owned(),
        button_style(palette.accent, palette.accent_text, palette.accent),
    );
    classes.insert(
        "gradient_strip".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 6.0,
            height: Some(24.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "gradient_stop".to_owned(),
        UiStyle {
            background: Some(palette.text_muted),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 999.0,
            width: Some(12.0),
            height: Some(28.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "gradient_stop_selected".to_owned(),
        UiStyle {
            background: Some(palette.accent),
            border_color: Some(palette.accent_text),
            border_width: 2.0,
            border_radius: 999.0,
            width: Some(14.0),
            height: Some(30.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "preview_panel".to_owned(),
        UiStyle {
            background: Some(palette.surface),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 12.0,
            padding: 16.0,
            gap: 10.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "preview_viewport".to_owned(),
        UiStyle {
            background: Some(palette.background),
            border_color: Some(palette.accent),
            border_width: 1.0,
            border_radius: 8.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "toast".to_owned(),
        UiStyle {
            background: Some(palette.accent),
            color: Some(palette.accent_text),
            border_color: Some(palette.accent),
            border_width: 1.0,
            border_radius: 999.0,
            padding: 8.0,
            font_size: 14.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "badge".to_owned(),
        UiStyle {
            background: Some(palette.accent),
            color: Some(palette.accent_text),
            border_radius: 999.0,
            padding: 6.0,
            font_size: 13.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "divider".to_owned(),
        UiStyle {
            background: Some(palette.border),
            height: Some(1.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "debug_text".to_owned(),
        UiStyle {
            color: Some(palette.text_muted),
            font_size: 13.0,
            ..UiStyle::default()
        },
    );
    classes
}

fn button_style(background: ColorRgba, color: ColorRgba, border: ColorRgba) -> UiStyle {
    UiStyle {
        background: Some(background),
        color: Some(color),
        border_color: Some(border),
        border_width: 1.0,
        border_radius: 8.0,
        padding: 10.0,
        font_size: 16.0,
        ..UiStyle::default()
    }
}
