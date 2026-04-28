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
            target: UiTarget::ScreenSpace { layer },
            root,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiTarget {
    ScreenSpace { layer: UiLayer },
}

impl UiTarget {
    pub fn layer(&self) -> UiLayer {
        match self {
            Self::ScreenSpace { layer } => *layer,
        }
    }
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
    pub style: UiStyle,
    pub events: UiEvents,
    pub children: Vec<UiNode>,
}

impl UiNode {
    pub fn new(kind: UiNodeKind) -> Self {
        Self {
            id: None,
            kind,
            style: UiStyle::default(),
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

    pub fn with_children(mut self, children: Vec<UiNode>) -> Self {
        self.children = children;
        self
    }

    pub fn with_events(mut self, events: UiEvents) -> Self {
        self.events = events;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiNodeKind {
    Panel,
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
    Spacer,
}

impl UiNodeKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Panel => "panel",
            Self::Row => "row",
            Self::Column => "column",
            Self::Stack => "stack",
            Self::Text { .. } => "text",
            Self::Button { .. } => "button",
            Self::ProgressBar { .. } => "progress-bar",
            Self::Spacer => "spacer",
        }
    }
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
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiEvents {
    pub on_click: Option<UiEventBinding>,
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
