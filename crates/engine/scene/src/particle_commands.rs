use std::collections::BTreeMap;
use std::path::PathBuf;

use amigo_fx::ColorRamp;
use amigo_math::{ColorRgba, Curve1d, Vec2};

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectileEmitter2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub pool: String,
    pub speed: f32,
    pub spawn_offset: Vec2,
    pub inherit_velocity_scale: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputActionMapSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub id: String,
    pub active: bool,
    pub actions: BTreeMap<String, InputActionBindingSceneCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputActionBindingSceneCommand {
    Axis {
        positive: Vec<String>,
        negative: Vec<String>,
    },
    Button {
        pressed: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub condition: Option<BehaviorConditionSceneCommand>,
    pub behavior: BehaviorKindSceneCommand,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorConditionSceneCommand {
    pub state_key: String,
    pub equals: Option<String>,
    pub not_equals: Option<String>,
    pub greater_than: Option<f64>,
    pub greater_or_equal: Option<f64>,
    pub less_than: Option<f64>,
    pub less_or_equal: Option<f64>,
    pub is_true: bool,
    pub is_false: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BehaviorKindSceneCommand {
    FreeflightInputController {
        target_entity: String,
        thrust_action: String,
        turn_action: String,
        strafe_action: Option<String>,
    },
    ParticleIntensityController {
        emitter: String,
        action: String,
    },
    ParticleProfileController {
        emitter: String,
        action: String,
        max_hold_seconds: f32,
        phases: Vec<ParticleProfilePhaseSceneCommand>,
    },
    CameraFollowModeController {
        camera: String,
        action: String,
        target: Option<String>,
        lerp: Option<f32>,
        lookahead_velocity_scale: Option<f32>,
        lookahead_max_distance: Option<f32>,
        sway_amount: Option<f32>,
        sway_frequency: Option<f32>,
    },
    ProjectileFireController {
        emitter: String,
        source: Option<String>,
        action: String,
        cooldown_seconds: f32,
        cooldown_id: Option<String>,
        audio: Option<String>,
    },
    MenuNavigationController {
        index_state: String,
        item_count: i64,
        item_count_state: Option<String>,
        up_action: String,
        down_action: String,
        confirm_action: Option<String>,
        wrap: bool,
        move_audio: Option<String>,
        confirm_audio: Option<String>,
        confirm_events: Vec<String>,
        selected_color_prefix: Option<String>,
        selected_color: String,
        unselected_color: String,
    },
    SceneTransitionController {
        action: String,
        scene: String,
    },
    SceneAutoTransitionController {
        scene: String,
    },
    SetStateOnActionController {
        action: String,
        key: String,
        value: String,
        audio: Option<String>,
    },
    ToggleStateController {
        action: String,
        key: String,
        default: bool,
        audio: Option<String>,
    },
    UiThemeSwitcher {
        bindings: BTreeMap<String, String>,
        cycle_action: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleProfilePhaseSceneCommand {
    pub id: String,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub velocity_mode: Option<ParticleProfileVelocityModeSceneCommand>,
    pub color_ramp: Option<ColorRamp>,
    pub spawn_rate: Option<ParticleProfileScalarSceneCommand>,
    pub lifetime: Option<ParticleProfileScalarSceneCommand>,
    pub lifetime_jitter: Option<ParticleProfileScalarSceneCommand>,
    pub speed: Option<ParticleProfileScalarSceneCommand>,
    pub speed_jitter: Option<ParticleProfileScalarSceneCommand>,
    pub spread_degrees: Option<ParticleProfileScalarSceneCommand>,
    pub initial_size: Option<ParticleProfileScalarSceneCommand>,
    pub final_size: Option<ParticleProfileScalarSceneCommand>,
    pub spawn_area_line: Option<ParticleProfileScalarSceneCommand>,
    pub shape_line: Option<ParticleProfileScalarSceneCommand>,
    pub shape_circle_weight: Option<ParticleProfileScalarSceneCommand>,
    pub shape_line_weight: Option<ParticleProfileScalarSceneCommand>,
    pub shape_quad_weight: Option<ParticleProfileScalarSceneCommand>,
    pub size_curve: Option<ParticleProfileCurve4SceneCommand>,
    pub speed_curve: Option<ParticleProfileCurve4SceneCommand>,
    pub alpha_curve: Option<ParticleProfileCurve4SceneCommand>,
    pub burst: Option<ParticleProfileBurstSceneCommand>,
    pub clear_forces: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleProfileVelocityModeSceneCommand {
    Free,
    SourceInertial,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleProfileScalarSceneCommand {
    pub from: f32,
    pub to: f32,
    pub curve: Curve1d,
    pub intensity_scale: f32,
    pub noise_scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleProfileCurve4SceneCommand {
    pub v0: ParticleProfileScalarSceneCommand,
    pub v1: ParticleProfileScalarSceneCommand,
    pub v2: ParticleProfileScalarSceneCommand,
    pub v3: ParticleProfileScalarSceneCommand,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleProfileBurstSceneCommand {
    pub rate_hz: f32,
    pub min_count: usize,
    pub max_count: usize,
    pub threshold: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventPipelineSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub id: String,
    pub topic: String,
    pub steps: Vec<EventPipelineStepSceneCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventPipelineStepSceneCommand {
    PlayAudio { clip: String },
    SetState { key: String, value: String },
    IncrementState { key: String, by: f64 },
    ShowUi { path: String },
    HideUi { path: String },
    BurstParticles { emitter: String, count: usize },
    TransitionScene { scene: String },
    EmitEvent { topic: String, payload: Vec<String> },
    Script { function: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiModelBindingsSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub bindings: Vec<UiModelBindingSceneCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiModelBindingSceneCommand {
    pub path: String,
    pub state_key: String,
    pub kind: UiModelBindingKindSceneCommand,
    pub format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiModelBindingKindSceneCommand {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptComponentSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub script: PathBuf,
    pub params: BTreeMap<String, ScriptComponentParamValueSceneCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptComponentParamValueSceneCommand {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleShape2dSceneCommand {
    Circle { segments: u32 },
    Quad,
    Line { length: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleLineAnchor2dSceneCommand {
    Center,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleShapeChoice2dSceneCommand {
    pub shape: ParticleShape2dSceneCommand,
    pub weight: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleShapeKeyframe2dSceneCommand {
    pub t: f32,
    pub shape: ParticleShape2dSceneCommand,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleSpawnArea2dSceneCommand {
    Point,
    Line {
        length: f32,
    },
    Rect {
        size: Vec2,
    },
    Circle {
        radius: f32,
    },
    Ring {
        inner_radius: f32,
        outer_radius: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleForce2dSceneCommand {
    Gravity { acceleration: Vec2 },
    ConstantAcceleration { acceleration: Vec2 },
    Drag { coefficient: f32 },
    Wind { velocity: Vec2, strength: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleVelocityMode2dSceneCommand {
    Free,
    SourceInertial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleSimulationSpace2dSceneCommand {
    World,
    Source,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleAlignMode2dSceneCommand {
    None,
    Velocity,
    Emitter,
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleBlendMode2dSceneCommand {
    Alpha,
    Additive,
    Multiply,
    Screen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleMotionStretch2dSceneCommand {
    pub enabled: bool,
    pub velocity_scale: f32,
    pub max_length: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleMaterial2dSceneCommand {
    pub receives_light: bool,
    pub light_response: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleLight2dSceneCommand {
    pub radius: f32,
    pub intensity: f32,
    pub mode: ParticleLightMode2dSceneCommand,
    pub glow: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleLightMode2dSceneCommand {
    Source,
    Particle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleEmitter2dSceneCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub attached_to: Option<String>,
    pub local_offset: Vec2,
    pub local_direction_radians: f32,
    pub spawn_area: ParticleSpawnArea2dSceneCommand,
    pub active: bool,
    pub spawn_rate: f32,
    pub max_particles: usize,
    pub particle_lifetime: f32,
    pub lifetime_jitter: f32,
    pub initial_speed: f32,
    pub speed_jitter: f32,
    pub spread_radians: f32,
    pub inherit_parent_velocity: f32,
    pub velocity_mode: ParticleVelocityMode2dSceneCommand,
    pub simulation_space: ParticleSimulationSpace2dSceneCommand,
    pub initial_size: f32,
    pub final_size: f32,
    pub color: ColorRgba,
    pub color_ramp: Option<ColorRamp>,
    pub z_index: f32,
    pub shape: ParticleShape2dSceneCommand,
    pub shape_choices: Vec<ParticleShapeChoice2dSceneCommand>,
    pub shape_over_lifetime: Vec<ParticleShapeKeyframe2dSceneCommand>,
    pub line_anchor: ParticleLineAnchor2dSceneCommand,
    pub align: ParticleAlignMode2dSceneCommand,
    pub blend_mode: ParticleBlendMode2dSceneCommand,
    pub motion_stretch: Option<ParticleMotionStretch2dSceneCommand>,
    pub material: ParticleMaterial2dSceneCommand,
    pub light: Option<ParticleLight2dSceneCommand>,
    pub emission_rate_curve: Curve1d,
    pub size_curve: Curve1d,
    pub alpha_curve: Curve1d,
    pub speed_curve: Curve1d,
    pub forces: Vec<ParticleForce2dSceneCommand>,
}

impl ProjectileEmitter2dSceneCommand {
    pub fn new(
        source_mod: impl Into<String>,
        entity_name: impl Into<String>,
        pool: impl Into<String>,
        speed: f32,
        spawn_offset: Vec2,
        inherit_velocity_scale: f32,
    ) -> Self {
        Self {
            source_mod: source_mod.into(),
            entity_name: entity_name.into(),
            pool: pool.into(),
            speed,
            spawn_offset,
            inherit_velocity_scale,
        }
    }
}

