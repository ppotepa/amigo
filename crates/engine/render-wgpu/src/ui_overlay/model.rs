use amigo_assets::AssetKey;
use amigo_math::ColorRgba;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UiOverlayLayer {
    Background,
    Hud,
    Menu,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiViewportSize {
    pub width: f32,
    pub height: f32,
}

impl UiViewportSize {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiOverlayDocument {
    pub entity_name: String,
    pub layer: UiOverlayLayer,
    pub viewport: Option<UiOverlayViewport>,
    pub root: UiOverlayNode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiOverlayViewport {
    pub width: f32,
    pub height: f32,
    pub scaling: UiOverlayViewportScaling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiOverlayViewportScaling {
    Expand,
    Fixed,
    Fit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiOverlayNode {
    pub id: Option<String>,
    pub kind: UiOverlayNodeKind,
    pub style: UiOverlayStyle,
    pub children: Vec<UiOverlayNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiOverlayNodeKind {
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
        expanded: bool,
        scroll_offset: f32,
        font: Option<AssetKey>,
    },
    TabView {
        selected: String,
        tabs: Vec<UiOverlayTab>,
        font: Option<AssetKey>,
    },
    ColorPickerRgb {
        color: ColorRgba,
    },
    CurveEditor {
        points: Vec<UiOverlayCurvePoint>,
    },
    Spacer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiOverlayCurvePoint {
    pub t: f32,
    pub value: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiOverlayTab {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiOverlayStyle {
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
    pub text_anchor: UiTextAnchor,
}

impl Default for UiOverlayStyle {
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
            text_anchor: UiTextAnchor::TopLeft,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub fn inset(self, inset: f32) -> Self {
        let clamped = inset.max(0.0).min(self.width * 0.5).min(self.height * 0.5);
        Self {
            x: self.x + clamped,
            y: self.y + clamped,
            width: (self.width - clamped * 2.0).max(0.0),
            height: (self.height - clamped * 2.0).max(0.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiLayoutNode {
    pub path: String,
    pub rect: UiRect,
    pub node: UiOverlayNode,
    pub children: Vec<UiLayoutNode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTextAnchor {
    TopLeft,
    Center,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiDrawPrimitive {
    Quad {
        rect: UiRect,
        color: ColorRgba,
    },
    Text {
        rect: UiRect,
        content: String,
        color: ColorRgba,
        font_size: f32,
        font: Option<AssetKey>,
        anchor: UiTextAnchor,
        word_wrap: bool,
        fit_to_width: bool,
    },
    ProgressBar {
        rect: UiRect,
        value: f32,
        background: ColorRgba,
        foreground: ColorRgba,
    },
}

