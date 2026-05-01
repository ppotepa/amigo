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
