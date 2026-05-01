use amigo_fx::ColorRamp;
use amigo_math::{ColorRgba, Curve1d, Transform2, Vec2};
use amigo_scene::{
    ParticleAlignMode2dSceneCommand, ParticleBlendMode2dSceneCommand,
    ParticleEmitter2dSceneCommand, ParticleForce2dSceneCommand, ParticleLightMode2dSceneCommand,
    ParticleLineAnchor2dSceneCommand, ParticleMotionStretch2dSceneCommand,
    ParticleShape2dSceneCommand, ParticleShapeChoice2dSceneCommand,
    ParticleShapeKeyframe2dSceneCommand, ParticleSimulationSpace2dSceneCommand,
    ParticleSpawnArea2dSceneCommand, ParticleVelocityMode2dSceneCommand, SceneEntityId,
};

pub const PARTICLES_2D_PLUGIN_LABEL: &str = "amigo-2d-particles";
pub const PARTICLES_2D_CAPABILITY: &str = "particles_2d";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleShape2d {
    Circle { segments: u32 },
    Quad,
    Line { length: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeightedParticleShape2d {
    pub shape: ParticleShape2d,
    pub weight: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleShapeKeyframe2d {
    pub t: f32,
    pub shape: ParticleShape2d,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleSpawnArea2d {
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
pub enum ParticleForce2d {
    Gravity { acceleration: Vec2 },
    ConstantAcceleration { acceleration: Vec2 },
    Drag { coefficient: f32 },
    Wind { velocity: Vec2, strength: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleVelocityMode2d {
    Free,
    SourceInertial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleSimulationSpace2d {
    World,
    Source,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleAlignMode2d {
    None,
    Velocity,
    Emitter,
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleLineAnchor2d {
    Center,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleBlendMode2d {
    Alpha,
    Additive,
    Multiply,
    Screen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleMotionStretch2d {
    pub enabled: bool,
    pub velocity_scale: f32,
    pub max_length: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleMaterial2d {
    pub receives_light: bool,
    pub light_response: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleLight2d {
    pub radius: f32,
    pub intensity: f32,
    pub mode: ParticleLightMode2d,
    pub glow: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleLightMode2d {
    Source,
    Particle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleEmitter2d {
    pub attached_to: Option<String>,
    pub local_offset: Vec2,
    pub local_direction_radians: f32,
    pub spawn_area: ParticleSpawnArea2d,
    pub active: bool,
    pub spawn_rate: f32,
    pub max_particles: usize,
    pub particle_lifetime: f32,
    pub lifetime_jitter: f32,
    pub initial_speed: f32,
    pub speed_jitter: f32,
    pub spread_radians: f32,
    pub inherit_parent_velocity: f32,
    pub velocity_mode: ParticleVelocityMode2d,
    pub simulation_space: ParticleSimulationSpace2d,
    pub initial_size: f32,
    pub final_size: f32,
    pub color: ColorRgba,
    pub color_ramp: Option<ColorRamp>,
    pub z_index: f32,
    pub shape: ParticleShape2d,
    pub shape_choices: Vec<WeightedParticleShape2d>,
    pub shape_over_lifetime: Vec<ParticleShapeKeyframe2d>,
    pub line_anchor: ParticleLineAnchor2d,
    pub align: ParticleAlignMode2d,
    pub blend_mode: ParticleBlendMode2d,
    pub motion_stretch: Option<ParticleMotionStretch2d>,
    pub material: ParticleMaterial2d,
    pub light: Option<ParticleLight2d>,
    pub emission_rate_curve: Curve1d,
    pub size_curve: Curve1d,
    pub alpha_curve: Curve1d,
    pub speed_curve: Curve1d,
    pub forces: Vec<ParticleForce2d>,
}

impl ParticleEmitter2d {
    pub fn from_scene_command(command: &ParticleEmitter2dSceneCommand) -> Self {
        Self {
            attached_to: command.attached_to.clone(),
            local_offset: command.local_offset,
            local_direction_radians: command.local_direction_radians,
            spawn_area: particle_spawn_area_from_scene_command(command.spawn_area),
            active: command.active,
            spawn_rate: command.spawn_rate,
            max_particles: command.max_particles,
            particle_lifetime: command.particle_lifetime,
            lifetime_jitter: command.lifetime_jitter,
            initial_speed: command.initial_speed,
            speed_jitter: command.speed_jitter,
            spread_radians: command.spread_radians,
            inherit_parent_velocity: command.inherit_parent_velocity,
            velocity_mode: particle_velocity_mode_from_scene_command(command.velocity_mode),
            simulation_space: particle_simulation_space_from_scene_command(
                command.simulation_space,
            ),
            initial_size: command.initial_size,
            final_size: command.final_size,
            color: command.color,
            color_ramp: command.color_ramp.clone(),
            z_index: command.z_index,
            shape: particle_shape_from_scene_command(command.shape),
            shape_choices: command
                .shape_choices
                .iter()
                .copied()
                .map(particle_shape_choice_from_scene_command)
                .filter(|choice| choice.weight > 0.0)
                .collect(),
            shape_over_lifetime: command
                .shape_over_lifetime
                .iter()
                .copied()
                .map(particle_shape_keyframe_from_scene_command)
                .collect(),
            line_anchor: particle_line_anchor_from_scene_command(command.line_anchor),
            align: particle_align_from_scene_command(command.align),
            blend_mode: particle_blend_from_scene_command(command.blend_mode),
            motion_stretch: command
                .motion_stretch
                .map(particle_motion_stretch_from_scene_command),
            material: ParticleMaterial2d {
                receives_light: command.material.receives_light,
                light_response: command.material.light_response.max(0.0),
            },
            light: command.light.map(|light| ParticleLight2d {
                radius: light.radius.max(0.0),
                intensity: light.intensity.max(0.0),
                mode: particle_light_mode_from_scene_command(light.mode),
                glow: light.glow,
            }),
            emission_rate_curve: command.emission_rate_curve.clone(),
            size_curve: command.size_curve.clone(),
            alpha_curve: command.alpha_curve.clone(),
            speed_curve: command.speed_curve.clone(),
            forces: command
                .forces
                .iter()
                .copied()
                .map(particle_force_from_scene_command)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticlePreset2d {
    pub source_mod: String,
    pub id: String,
    pub label: String,
    pub category: String,
    pub tags: Vec<String>,
    pub emitter: ParticleEmitter2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleEmitter2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub emitter: ParticleEmitter2d,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle2d {
    pub previous_position: Vec2,
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation_radians: f32,
    pub shape: ParticleShape2d,
    pub age: f32,
    pub lifetime: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Particle2dDrawCommand {
    pub emitter_entity_name: String,
    pub previous_position: Vec2,
    pub position: Vec2,
    pub size: f32,
    pub color: ColorRgba,
    pub z_index: f32,
    pub shape: ParticleShape2d,
    pub line_anchor: ParticleLineAnchor2d,
    pub blend_mode: ParticleBlendMode2d,
    pub motion_stretch: Option<ParticleMotionStretch2d>,
    pub material: ParticleMaterial2d,
    pub light: Option<ParticleLight2d>,
    pub light_position: Option<Vec2>,
    pub transform: Transform2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Particle2dEmitterRuntimeInput {
    pub emitter_entity_name: String,
    pub source_entity_name: String,
    pub source_transform: Transform2,
    pub source_velocity: Vec2,
    pub source_visible: bool,
    pub source_simulation_enabled: bool,
}

fn particle_shape_from_scene_command(shape: ParticleShape2dSceneCommand) -> ParticleShape2d {
    match shape {
        ParticleShape2dSceneCommand::Circle { segments } => ParticleShape2d::Circle { segments },
        ParticleShape2dSceneCommand::Quad => ParticleShape2d::Quad,
        ParticleShape2dSceneCommand::Line { length } => ParticleShape2d::Line { length },
    }
}

fn particle_shape_choice_from_scene_command(
    choice: ParticleShapeChoice2dSceneCommand,
) -> WeightedParticleShape2d {
    WeightedParticleShape2d {
        shape: particle_shape_from_scene_command(choice.shape),
        weight: choice.weight.max(0.0),
    }
}

fn particle_shape_keyframe_from_scene_command(
    keyframe: ParticleShapeKeyframe2dSceneCommand,
) -> ParticleShapeKeyframe2d {
    ParticleShapeKeyframe2d {
        t: keyframe.t.clamp(0.0, 1.0),
        shape: particle_shape_from_scene_command(keyframe.shape),
    }
}

fn particle_align_from_scene_command(
    align: ParticleAlignMode2dSceneCommand,
) -> ParticleAlignMode2d {
    match align {
        ParticleAlignMode2dSceneCommand::None => ParticleAlignMode2d::None,
        ParticleAlignMode2dSceneCommand::Velocity => ParticleAlignMode2d::Velocity,
        ParticleAlignMode2dSceneCommand::Emitter => ParticleAlignMode2d::Emitter,
        ParticleAlignMode2dSceneCommand::Random => ParticleAlignMode2d::Random,
    }
}

fn particle_line_anchor_from_scene_command(
    anchor: ParticleLineAnchor2dSceneCommand,
) -> ParticleLineAnchor2d {
    match anchor {
        ParticleLineAnchor2dSceneCommand::Center => ParticleLineAnchor2d::Center,
        ParticleLineAnchor2dSceneCommand::Start => ParticleLineAnchor2d::Start,
        ParticleLineAnchor2dSceneCommand::End => ParticleLineAnchor2d::End,
    }
}

fn particle_blend_from_scene_command(
    blend_mode: ParticleBlendMode2dSceneCommand,
) -> ParticleBlendMode2d {
    match blend_mode {
        ParticleBlendMode2dSceneCommand::Alpha => ParticleBlendMode2d::Alpha,
        ParticleBlendMode2dSceneCommand::Additive => ParticleBlendMode2d::Additive,
        ParticleBlendMode2dSceneCommand::Multiply => ParticleBlendMode2d::Multiply,
        ParticleBlendMode2dSceneCommand::Screen => ParticleBlendMode2d::Screen,
    }
}

fn particle_velocity_mode_from_scene_command(
    velocity_mode: ParticleVelocityMode2dSceneCommand,
) -> ParticleVelocityMode2d {
    match velocity_mode {
        ParticleVelocityMode2dSceneCommand::Free => ParticleVelocityMode2d::Free,
        ParticleVelocityMode2dSceneCommand::SourceInertial => {
            ParticleVelocityMode2d::SourceInertial
        }
    }
}

fn particle_simulation_space_from_scene_command(
    simulation_space: ParticleSimulationSpace2dSceneCommand,
) -> ParticleSimulationSpace2d {
    match simulation_space {
        ParticleSimulationSpace2dSceneCommand::World => ParticleSimulationSpace2d::World,
        ParticleSimulationSpace2dSceneCommand::Source => ParticleSimulationSpace2d::Source,
    }
}

fn particle_light_mode_from_scene_command(
    mode: ParticleLightMode2dSceneCommand,
) -> ParticleLightMode2d {
    match mode {
        ParticleLightMode2dSceneCommand::Source => ParticleLightMode2d::Source,
        ParticleLightMode2dSceneCommand::Particle => ParticleLightMode2d::Particle,
    }
}

fn particle_motion_stretch_from_scene_command(
    stretch: ParticleMotionStretch2dSceneCommand,
) -> ParticleMotionStretch2d {
    ParticleMotionStretch2d {
        enabled: stretch.enabled,
        velocity_scale: stretch.velocity_scale.max(0.0),
        max_length: stretch.max_length.max(0.0),
    }
}

fn particle_spawn_area_from_scene_command(
    spawn_area: ParticleSpawnArea2dSceneCommand,
) -> ParticleSpawnArea2d {
    match spawn_area {
        ParticleSpawnArea2dSceneCommand::Point => ParticleSpawnArea2d::Point,
        ParticleSpawnArea2dSceneCommand::Line { length } => ParticleSpawnArea2d::Line { length },
        ParticleSpawnArea2dSceneCommand::Rect { size } => ParticleSpawnArea2d::Rect { size },
        ParticleSpawnArea2dSceneCommand::Circle { radius } => {
            ParticleSpawnArea2d::Circle { radius }
        }
        ParticleSpawnArea2dSceneCommand::Ring {
            inner_radius,
            outer_radius,
        } => ParticleSpawnArea2d::Ring {
            inner_radius,
            outer_radius,
        },
    }
}

fn particle_force_from_scene_command(force: ParticleForce2dSceneCommand) -> ParticleForce2d {
    match force {
        ParticleForce2dSceneCommand::Gravity { acceleration } => {
            ParticleForce2d::Gravity { acceleration }
        }
        ParticleForce2dSceneCommand::ConstantAcceleration { acceleration } => {
            ParticleForce2d::ConstantAcceleration { acceleration }
        }
        ParticleForce2dSceneCommand::Drag { coefficient } => ParticleForce2d::Drag { coefficient },
        ParticleForce2dSceneCommand::Wind { velocity, strength } => {
            ParticleForce2d::Wind { velocity, strength }
        }
    }
}
