#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorCommand {
    pub source_mod: String,
    pub entity_name: String,
    pub condition: Option<BehaviorCondition>,
    pub behavior: BehaviorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorCondition {
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
pub enum BehaviorKind {
    FreeflightInputController(FreeflightInputControllerBehavior),
    CameraFollowModeController(CameraFollowModeControllerBehavior),
    ParticleIntensityController(ParticleIntensityControllerBehavior),
    ParticleProfileController(ParticleProfileControllerBehavior),
    ProjectileFireController(ProjectileFireControllerBehavior),
    MenuNavigationController(MenuNavigationControllerBehavior),
    SceneAutoTransitionController(SceneAutoTransitionControllerBehavior),
    SceneTransitionController(SceneTransitionControllerBehavior),
    SetStateOnActionController(SetStateOnActionControllerBehavior),
    ToggleStateController(ToggleStateControllerBehavior),
    UiThemeSwitcher(UiThemeSwitcherBehavior),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreeflightInputControllerBehavior {
    pub target_entity: String,
    pub thrust_action: String,
    pub turn_action: String,
    pub strafe_action: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraFollowModeControllerBehavior {
    pub camera: String,
    pub action: String,
    pub target: Option<String>,
    pub lerp: Option<f32>,
    pub lookahead_velocity_scale: Option<f32>,
    pub lookahead_max_distance: Option<f32>,
    pub sway_amount: Option<f32>,
    pub sway_frequency: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleIntensityControllerBehavior {
    pub emitter: String,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleProfileControllerBehavior {
    pub emitter: String,
    pub action: String,
    pub max_hold_seconds: f32,
    pub phases: Vec<ParticleProfilePhase>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleProfilePhase {
    pub id: String,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub velocity_mode: Option<ParticleProfileVelocityMode>,
    pub color_ramp: Option<ColorRamp>,
    pub spawn_rate: Option<ParticleProfileScalar>,
    pub lifetime: Option<ParticleProfileScalar>,
    pub lifetime_jitter: Option<ParticleProfileScalar>,
    pub speed: Option<ParticleProfileScalar>,
    pub speed_jitter: Option<ParticleProfileScalar>,
    pub spread_degrees: Option<ParticleProfileScalar>,
    pub initial_size: Option<ParticleProfileScalar>,
    pub final_size: Option<ParticleProfileScalar>,
    pub spawn_area_line: Option<ParticleProfileScalar>,
    pub shape_line: Option<ParticleProfileScalar>,
    pub shape_circle_weight: Option<ParticleProfileScalar>,
    pub shape_line_weight: Option<ParticleProfileScalar>,
    pub shape_quad_weight: Option<ParticleProfileScalar>,
    pub size_curve: Option<ParticleProfileCurve4>,
    pub speed_curve: Option<ParticleProfileCurve4>,
    pub alpha_curve: Option<ParticleProfileCurve4>,
    pub burst: Option<ParticleProfileBurst>,
    pub clear_forces: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleProfileVelocityMode {
    Free,
    SourceInertial,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleProfileScalar {
    pub from: f32,
    pub to: f32,
    pub curve: Curve1d,
    pub intensity_scale: f32,
    pub noise_scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleProfileCurve4 {
    pub v0: ParticleProfileScalar,
    pub v1: ParticleProfileScalar,
    pub v2: ParticleProfileScalar,
    pub v3: ParticleProfileScalar,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleProfileBurst {
    pub rate_hz: f32,
    pub min_count: usize,
    pub max_count: usize,
    pub threshold: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectileFireControllerBehavior {
    pub emitter: String,
    pub source: Option<String>,
    pub action: String,
    pub cooldown_seconds: f32,
    pub cooldown_id: Option<String>,
    pub audio: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneTransitionControllerBehavior {
    pub action: String,
    pub scene: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneAutoTransitionControllerBehavior {
    pub scene: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuNavigationControllerBehavior {
    pub index_state: String,
    pub item_count: i64,
    pub item_count_state: Option<String>,
    pub up_action: String,
    pub down_action: String,
    pub confirm_action: Option<String>,
    pub wrap: bool,
    pub move_audio: Option<String>,
    pub confirm_audio: Option<String>,
    pub confirm_events: Vec<String>,
    pub selected_color_prefix: Option<String>,
    pub selected_color: String,
    pub unselected_color: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiThemeSwitcherBehavior {
    pub bindings: BTreeMap<String, String>,
    pub cycle_action: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetStateOnActionControllerBehavior {
    pub action: String,
    pub key: String,
    pub value: String,
    pub audio: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToggleStateControllerBehavior {
    pub action: String,
    pub key: String,
    pub default: bool,
    pub audio: Option<String>,
}

#[derive(Debug, Default)]
pub struct BehaviorSceneService {
    behaviors: Mutex<BTreeMap<String, BehaviorCommand>>,
    hold_seconds: Mutex<BTreeMap<String, f32>>,
}
