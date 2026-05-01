use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::core::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SceneInputActionBindingDocument {
    Axis {
        #[serde(default)]
        positive: Vec<String>,
        #[serde(default)]
        negative: Vec<String>,
    },
    Button {
        #[serde(default)]
        pressed: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SceneBehaviorDocument {
    FreeflightInputController {
        target: String,
        input: SceneFreeflightInputActionsDocument,
    },
    ParticleIntensityController {
        emitter: String,
        action: String,
    },
    ParticleProfileController {
        emitter: String,
        action: String,
        #[serde(default = "default_particle_profile_max_hold_seconds")]
        max_hold_seconds: f32,
        #[serde(default)]
        phases: Vec<SceneParticleProfilePhaseDocument>,
    },
    CameraFollowModeController {
        camera: String,
        action: String,
        #[serde(default)]
        target: Option<String>,
        #[serde(default)]
        lerp: Option<f32>,
        #[serde(default)]
        lookahead_velocity_scale: Option<f32>,
        #[serde(default)]
        lookahead_max_distance: Option<f32>,
        #[serde(default)]
        sway_amount: Option<f32>,
        #[serde(default)]
        sway_frequency: Option<f32>,
    },
    ProjectileFireController {
        emitter: String,
        #[serde(default)]
        source: Option<String>,
        action: String,
        #[serde(default)]
        cooldown: f32,
        #[serde(default)]
        cooldown_id: Option<String>,
        #[serde(default)]
        audio: Option<String>,
    },
    MenuNavigationController {
        index_state: String,
        item_count: i64,
        #[serde(default)]
        item_count_state: Option<String>,
        up_action: String,
        down_action: String,
        #[serde(default)]
        confirm_action: Option<String>,
        #[serde(default = "default_true")]
        wrap: bool,
        #[serde(default)]
        move_audio: Option<String>,
        #[serde(default)]
        confirm_audio: Option<String>,
        #[serde(default)]
        confirm_events: Vec<String>,
        #[serde(default)]
        selected_color_prefix: Option<String>,
        #[serde(default = "default_selected_color")]
        selected_color: String,
        #[serde(default = "default_unselected_color")]
        unselected_color: String,
    },
    SceneTransitionController {
        action: String,
        scene: String,
    },
    SceneAutoTransitionController {
        scene: String,
    },
    SceneBackController {
        action: String,
        scene: String,
    },
    SetStateOnActionController {
        action: String,
        key: String,
        value: String,
        #[serde(default)]
        audio: Option<String>,
    },
    ToggleStateController {
        action: String,
        key: String,
        #[serde(default)]
        default: bool,
        #[serde(default)]
        audio: Option<String>,
    },
    UiThemeSwitcher {
        bindings: BTreeMap<String, String>,
        #[serde(default)]
        cycle: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneParticleProfilePhaseDocument {
    pub id: String,
    #[serde(default)]
    pub start_seconds: f32,
    pub end_seconds: f32,
    #[serde(default)]
    pub velocity_mode: Option<SceneParticleProfileVelocityModeDocument>,
    #[serde(default)]
    pub color_ramp: Option<ColorRampSceneDocument>,
    #[serde(default)]
    pub spawn_rate: Option<SceneParticleProfileScalarDocument>,
    #[serde(default)]
    pub lifetime: Option<SceneParticleProfileScalarDocument>,
    #[serde(default)]
    pub lifetime_jitter: Option<SceneParticleProfileScalarDocument>,
    #[serde(default)]
    pub speed: Option<SceneParticleProfileScalarDocument>,
    #[serde(default)]
    pub speed_jitter: Option<SceneParticleProfileScalarDocument>,
    #[serde(default)]
    pub spread_degrees: Option<SceneParticleProfileScalarDocument>,
    #[serde(default)]
    pub initial_size: Option<SceneParticleProfileScalarDocument>,
    #[serde(default)]
    pub final_size: Option<SceneParticleProfileScalarDocument>,
    #[serde(default)]
    pub spawn_area_line: Option<SceneParticleProfileScalarDocument>,
    #[serde(default)]
    pub shape_line: Option<SceneParticleProfileScalarDocument>,
    #[serde(default)]
    pub shape_circle_weight: Option<SceneParticleProfileScalarDocument>,
    #[serde(default)]
    pub shape_line_weight: Option<SceneParticleProfileScalarDocument>,
    #[serde(default)]
    pub shape_quad_weight: Option<SceneParticleProfileScalarDocument>,
    #[serde(default)]
    pub size_curve: Option<SceneParticleProfileCurve4Document>,
    #[serde(default)]
    pub speed_curve: Option<SceneParticleProfileCurve4Document>,
    #[serde(default)]
    pub alpha_curve: Option<SceneParticleProfileCurve4Document>,
    #[serde(default)]
    pub burst: Option<SceneParticleProfileBurstDocument>,
    #[serde(default)]
    pub clear_forces: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneParticleProfileScalarDocument {
    pub from: f32,
    pub to: f32,
    #[serde(default)]
    pub curve: Option<Curve1dSceneDocument>,
    #[serde(default)]
    pub intensity_scale: f32,
    #[serde(default)]
    pub noise_scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneParticleProfileCurve4Document {
    pub v0: SceneParticleProfileScalarDocument,
    pub v1: SceneParticleProfileScalarDocument,
    pub v2: SceneParticleProfileScalarDocument,
    pub v3: SceneParticleProfileScalarDocument,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneParticleProfileBurstDocument {
    #[serde(default)]
    pub rate_hz: f32,
    #[serde(default)]
    pub min_count: usize,
    #[serde(default)]
    pub max_count: usize,
    #[serde(default)]
    pub threshold: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SceneParticleProfileVelocityModeDocument {
    Free,
    SourceInertial,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneBehaviorConditionDocument {
    pub state: String,
    #[serde(default)]
    pub equals: Option<String>,
    #[serde(default)]
    pub not_equals: Option<String>,
    #[serde(default)]
    pub greater_than: Option<f64>,
    #[serde(default)]
    pub greater_or_equal: Option<f64>,
    #[serde(default)]
    pub less_than: Option<f64>,
    #[serde(default)]
    pub less_or_equal: Option<f64>,
    #[serde(default)]
    pub is_true: bool,
    #[serde(default)]
    pub is_false: bool,
}

fn default_true() -> bool {
    true
}

fn default_particle_profile_max_hold_seconds() -> f32 {
    1.0
}

fn default_selected_color() -> String {
    "#FFFFFFFF".to_owned()
}

fn default_unselected_color() -> String {
    "#9A9A9AFF".to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneFreeflightInputActionsDocument {
    pub thrust: String,
    pub turn: String,
    #[serde(default)]
    pub strafe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SceneEventPipelineStepDocument {
    PlayAudio {
        clip: String,
    },
    SetState {
        key: String,
        value: String,
    },
    IncrementState {
        key: String,
        by: f64,
    },
    ShowUi {
        path: String,
    },
    HideUi {
        path: String,
    },
    BurstParticles {
        emitter: String,
        count: usize,
    },
    TransitionScene {
        scene: String,
    },
    EmitEvent {
        topic: String,
        #[serde(default)]
        payload: Vec<String>,
    },
    Script {
        function: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneUiModelBindingDocument {
    pub path: String,
    pub state: String,
    pub kind: SceneUiModelBindingKindDocument,
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SceneUiModelBindingKindDocument {
    Text,
    Value,
    Visible,
    Enabled,
    Selected,
    Options,
    Color,
    Background,
    Theme,
}
