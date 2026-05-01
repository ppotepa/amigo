use serde::{Deserialize, Serialize};

use super::defaults::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneUiThemeComponentDocument {
    pub id: String,
    pub palette: SceneUiThemePaletteComponentDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneUiThemePaletteComponentDocument {
    pub background: String,
    pub surface: String,
    pub surface_alt: String,
    pub text: String,
    pub text_muted: String,
    pub border: String,
    pub accent: String,
    pub accent_text: String,
    pub danger: String,
    pub warning: String,
    pub success: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SceneLifetimeExpirationOutcomeDocument {
    Hide,
    Disable,
    Despawn,
    ReturnToPool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SceneBoundsBehavior2dDocument {
    Bounce,
    Wrap,
    Hide,
    Despawn,
    Clamp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SceneVectorShapeKindComponentDocument {
    Polyline,
    Polygon,
    Circle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneUiTargetComponentDocument {
    #[serde(rename = "type")]
    pub kind: SceneUiTargetTypeComponentDocument,
    pub layer: SceneUiLayerComponentDocument,
    #[serde(default)]
    pub viewport: Option<SceneUiViewportComponentDocument>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SceneUiViewportComponentDocument {
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub scaling: SceneUiViewportScalingComponentDocument,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SceneUiViewportScalingComponentDocument {
    #[default]
    Expand,
    Fixed,
    Fit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SceneUiTargetTypeComponentDocument {
    ScreenSpace,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum SceneUiLayerComponentDocument {
    Background,
    Hud,
    Menu,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneUiNodeComponentDocument {
    #[serde(rename = "type")]
    pub kind: SceneUiNodeTypeComponentDocument,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub style_class: Option<String>,
    #[serde(default)]
    pub style: SceneUiStyleComponentDocument,
    #[serde(default)]
    pub children: Vec<SceneUiNodeComponentDocument>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub font: Option<String>,
    #[serde(default)]
    pub value: Option<f32>,
    #[serde(default)]
    pub min: Option<f32>,
    #[serde(default)]
    pub max: Option<f32>,
    #[serde(default)]
    pub step: Option<f32>,
    #[serde(default)]
    pub checked: Option<bool>,
    #[serde(default)]
    pub selected: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub tabs: Vec<SceneUiTabComponentDocument>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub points: Vec<SceneUiCurvePointComponentDocument>,
    #[serde(default)]
    pub values: Vec<f32>,
    #[serde(default)]
    pub text_bind: Option<String>,
    #[serde(default)]
    pub visible_bind: Option<String>,
    #[serde(default)]
    pub enabled_bind: Option<String>,
    #[serde(default)]
    pub value_bind: Option<String>,
    #[serde(default)]
    pub on_click: Option<SceneUiEventBindingComponentDocument>,
    #[serde(default)]
    pub on_change: Option<SceneUiEventBindingComponentDocument>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SceneUiNodeTypeComponentDocument {
    Panel,
    GroupBox,
    Row,
    Column,
    Stack,
    Text,
    Button,
    ProgressBar,
    Slider,
    Toggle,
    OptionSet,
    Dropdown,
    TabView,
    ColorPickerRgb,
    CurveEditor,
    Spacer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SceneUiCurvePointComponentDocument {
    pub t: f32,
    pub value: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneUiTabComponentDocument {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SceneUiStyleComponentDocument {
    #[serde(default)]
    pub left: Option<f32>,
    #[serde(default)]
    pub top: Option<f32>,
    #[serde(default)]
    pub right: Option<f32>,
    #[serde(default)]
    pub bottom: Option<f32>,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub height: Option<f32>,
    #[serde(default)]
    pub padding: f32,
    #[serde(default)]
    pub gap: f32,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub border_color: Option<String>,
    #[serde(default)]
    pub border_width: f32,
    #[serde(default)]
    pub border_radius: f32,
    #[serde(default = "default_ui_font_size")]
    pub font_size: f32,
    #[serde(default)]
    pub word_wrap: bool,
    #[serde(default)]
    pub fit_to_width: bool,
    #[serde(default)]
    pub align: Option<SceneUiTextAlignComponentDocument>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SceneUiTextAlignComponentDocument {
    Start,
    Center,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneUiEventBindingComponentDocument {
    pub event: String,
    #[serde(default)]
    pub payload: Vec<String>,
}
