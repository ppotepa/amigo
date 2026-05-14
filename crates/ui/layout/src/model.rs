#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutRect {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutViewport {
    pub width: f32,
    pub height: f32,
}

impl LayoutViewport {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutViewportScaling {
    Expand,
    Fixed,
    Fit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutTab {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutStyle {
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub padding: f32,
    pub gap: f32,
    pub border_width: f32,
    pub border_radius: f32,
    pub font_size: f32,
    pub word_wrap: bool,
    pub fit_to_width: bool,
}

impl Default for LayoutStyle {
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
            border_width: 0.0,
            border_radius: 0.0,
            font_size: 16.0,
            word_wrap: false,
            fit_to_width: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutLeafKind {
    Text { content: String },
    Button { text: String },
    ProgressBar,
    Slider,
    Toggle { text: String },
    OptionSet { option_count: usize },
    Dropdown { option_count: usize, expanded: bool },
    ColorPickerRgb,
    CurveEditor,
    Spacer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutKind {
    Panel,
    GroupBox {
        label: String,
    },
    Row,
    Column,
    Stack,
    TabView {
        selected: String,
        tabs: Vec<LayoutTab>,
    },
    Leaf(LayoutLeafKind),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutElement<T> {
    pub id: Option<String>,
    pub kind: LayoutKind,
    pub style: LayoutStyle,
    pub data: T,
    pub children: Vec<LayoutElement<T>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutNode<T> {
    pub path: String,
    pub rect: LayoutRect,
    pub data: T,
    pub children: Vec<LayoutNode<T>>,
}
