use std::collections::BTreeMap;

use amigo_assets::AssetKey;
use amigo_math::{ColorRgba, Transform3, Vec2};

use crate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SceneUiTarget {
    ScreenSpace {
        layer: SceneUiLayer,
        viewport: Option<SceneUiViewport>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneUiViewport {
    pub width: f32,
    pub height: f32,
    pub scaling: SceneUiViewportScaling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneUiViewportScaling {
    Expand,
    Fixed,
    Fit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SceneUiLayer {
    Background,
    Hud,
    Menu,
    Debug,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneUiDocument {
    pub target: SceneUiTarget,
    pub root: SceneUiNode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneUiNode {
    pub id: Option<String>,
    pub kind: SceneUiNodeKind,
    pub style_class: Option<String>,
    pub style: SceneUiStyle,
    pub binds: SceneUiBinds,
    pub on_click: Option<SceneUiEventBinding>,
    pub on_change: Option<SceneUiEventBinding>,
    pub children: Vec<SceneUiNode>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SceneUiBinds {
    pub text: Option<String>,
    pub visible: Option<String>,
    pub enabled: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneUiNodeKind {
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
        tabs: Vec<SceneUiTab>,
        font: Option<AssetKey>,
    },
    ColorPickerRgb {
        color: ColorRgba,
    },
    CurveEditor {
        points: Vec<SceneUiCurvePoint>,
    },
    Spacer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneUiCurvePoint {
    pub t: f32,
    pub value: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneUiTab {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneUiStyle {
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
    pub align: SceneUiTextAlign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneUiTextAlign {
    Start,
    Center,
}

impl Default for SceneUiStyle {
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
            align: SceneUiTextAlign::Start,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneUiEventBinding {
    pub event: String,
    pub payload: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub document: SceneUiDocument,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiThemeSetSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub active: Option<String>,
    pub themes: Vec<SceneUiTheme>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneUiTheme {
    pub id: String,
    pub palette: SceneUiThemePalette,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneUiThemePalette {
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

#[derive(Debug, Clone, PartialEq)]
pub struct AudioCueSceneCommand {
    pub source_mod: String,
    pub name: String,
    pub clip: AssetKey,
    pub min_interval: Option<f32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SceneEntityLifecycleOverride {
    pub visible: Option<bool>,
    pub simulation_enabled: Option<bool>,
    pub collision_enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivationEntrySceneCommand {
    pub target: EntitySelector,
    pub lifecycle: SceneEntityLifecycleOverride,
    pub transform: Option<Transform3>,
    pub velocity: Option<Vec2>,
    pub angular_velocity: Option<f32>,
    pub properties: BTreeMap<String, ScenePropertyValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivationSetSceneCommand {
    pub source_mod: String,
    pub id: String,
    pub entries: Vec<ActivationEntrySceneCommand>,
}

