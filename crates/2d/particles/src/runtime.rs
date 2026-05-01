use amigo_math::{Transform2, Vec2};

use crate::model::*;

pub(crate) fn spawn_particle(
    emitter: &ParticleEmitter2d,
    input: &Particle2dEmitterRuntimeInput,
    intensity: f32,
    seed: &mut u64,
) -> Particle2d {
    spawn_particle_at(emitter, input, intensity, seed, None)
}

pub(crate) fn spawn_particle_at(
    emitter: &ParticleEmitter2d,
    input: &Particle2dEmitterRuntimeInput,
    intensity: f32,
    seed: &mut u64,
    position_override: Option<Vec2>,
) -> Particle2d {
    let parent_rotation = input.source_transform.rotation_radians;
    let source_space = emitter.simulation_space == ParticleSimulationSpace2d::Source;
    let emitter_rotation = if source_space {
        emitter.local_direction_radians
    } else {
        parent_rotation + emitter.local_direction_radians
    };
    let area_offset = sample_spawn_area(emitter.spawn_area, seed);
    let position = position_override.unwrap_or_else(|| {
        if source_space {
            Vec2::new(
                emitter.local_offset.x + area_offset.x,
                emitter.local_offset.y + area_offset.y,
            )
        } else {
            let offset = rotate_vec2(emitter.local_offset, parent_rotation);
            let area_offset = rotate_vec2(area_offset, parent_rotation);
            Vec2::new(
                input.source_transform.translation.x + offset.x + area_offset.x,
                input.source_transform.translation.y + offset.y + area_offset.y,
            )
        }
    });
    let position = if source_space && position_override.is_some() {
        world_to_local_position(position, input.source_transform)
    } else {
        position
    };
    let spread = next_signed_unit(seed) * emitter.spread_radians * 0.5;
    let direction_angle = emitter_rotation + spread;
    let speed_jitter = next_signed_unit(seed) * emitter.speed_jitter.max(0.0);
    let speed =
        (emitter.initial_speed * emitter.speed_curve.sample(intensity) + speed_jitter).max(0.0);
    let lifetime_jitter = next_signed_unit(seed) * emitter.lifetime_jitter.max(0.0);
    let lifetime = (emitter.particle_lifetime + lifetime_jitter).max(0.001);
    let direction = Vec2::new(direction_angle.cos(), direction_angle.sin());
    let rotation_radians = match emitter.align {
        ParticleAlignMode2d::Random => next_unit(seed) * std::f32::consts::TAU,
        ParticleAlignMode2d::Emitter => emitter_rotation,
        ParticleAlignMode2d::None | ParticleAlignMode2d::Velocity => 0.0,
    };
    let shape = sample_particle_shape(emitter, seed);
    let inherited_velocity = match emitter.velocity_mode {
        ParticleVelocityMode2d::Free if source_space => {
            let velocity = rotate_vec2(input.source_velocity, -parent_rotation);
            Vec2::new(
                velocity.x * emitter.inherit_parent_velocity,
                velocity.y * emitter.inherit_parent_velocity,
            )
        }
        ParticleVelocityMode2d::Free => Vec2::new(
            input.source_velocity.x * emitter.inherit_parent_velocity,
            input.source_velocity.y * emitter.inherit_parent_velocity,
        ),
        ParticleVelocityMode2d::SourceInertial if source_space => {
            rotate_vec2(input.source_velocity, -parent_rotation)
        }
        ParticleVelocityMode2d::SourceInertial => input.source_velocity,
    };
    Particle2d {
        previous_position: position,
        position,
        velocity: Vec2::new(
            direction.x * speed + inherited_velocity.x,
            direction.y * speed + inherited_velocity.y,
        ),
        rotation_radians,
        shape,
        age: 0.0,
        lifetime,
    }
}

pub(crate) fn sample_particle_shape(
    emitter: &ParticleEmitter2d,
    seed: &mut u64,
) -> ParticleShape2d {
    if emitter.shape_choices.is_empty() {
        return emitter.shape;
    }
    let total_weight = emitter
        .shape_choices
        .iter()
        .map(|choice| choice.weight.max(0.0))
        .sum::<f32>();
    if total_weight <= f32::EPSILON {
        return emitter.shape;
    }
    let mut cursor = next_unit(seed) * total_weight;
    for choice in &emitter.shape_choices {
        cursor -= choice.weight.max(0.0);
        if cursor <= 0.0 {
            return choice.shape;
        }
    }
    emitter
        .shape_choices
        .last()
        .map(|choice| choice.shape)
        .unwrap_or(emitter.shape)
}

pub(crate) fn sample_particle_shape_over_lifetime(
    keyframes: &[ParticleShapeKeyframe2d],
    fallback: ParticleShape2d,
    age_t: f32,
) -> ParticleShape2d {
    if keyframes.is_empty() {
        return fallback;
    }
    let age_t = age_t.clamp(0.0, 1.0);
    let mut first = keyframes[0];
    let mut sampled: Option<ParticleShapeKeyframe2d> = None;
    for keyframe in keyframes {
        if keyframe.t < first.t {
            first = *keyframe;
        }
        if keyframe.t <= age_t {
            sampled = match sampled {
                Some(current) if current.t > keyframe.t => Some(current),
                _ => Some(*keyframe),
            };
        }
    }
    sampled.unwrap_or(first).shape
}

pub(crate) fn particle_position_for_draw(
    position: Vec2,
    source_transform: Option<Transform2>,
) -> Vec2 {
    source_transform
        .map(|transform| local_to_world_position(position, transform))
        .unwrap_or(position)
}

pub(crate) fn particle_rotation_for_align(
    particle: &Particle2d,
    align: ParticleAlignMode2d,
    source_transform: Option<Transform2>,
) -> f32 {
    let source_rotation = source_transform
        .map(|transform| transform.rotation_radians)
        .unwrap_or(0.0);
    match align {
        ParticleAlignMode2d::None => 0.0,
        ParticleAlignMode2d::Velocity => {
            particle.velocity.y.atan2(particle.velocity.x) + source_rotation
        }
        ParticleAlignMode2d::Emitter | ParticleAlignMode2d::Random => {
            particle.rotation_radians + source_rotation
        }
    }
}

pub(crate) fn apply_particle_forces(
    particle: &mut Particle2d,
    forces: &[ParticleForce2d],
    delta_seconds: f32,
) {
    let mut acceleration = Vec2::ZERO;
    for force in forces {
        match *force {
            ParticleForce2d::Gravity {
                acceleration: force_acceleration,
            }
            | ParticleForce2d::ConstantAcceleration {
                acceleration: force_acceleration,
            } => {
                acceleration = Vec2::new(
                    acceleration.x + force_acceleration.x,
                    acceleration.y + force_acceleration.y,
                );
            }
            ParticleForce2d::Drag { coefficient } => {
                let amount = coefficient.max(0.0) * delta_seconds;
                particle.velocity = Vec2::new(
                    move_towards(particle.velocity.x, 0.0, amount),
                    move_towards(particle.velocity.y, 0.0, amount),
                );
            }
            ParticleForce2d::Wind { velocity, strength } => {
                let k = (strength.max(0.0) * delta_seconds).clamp(0.0, 1.0);
                particle.velocity = Vec2::new(
                    particle.velocity.x + (velocity.x - particle.velocity.x) * k,
                    particle.velocity.y + (velocity.y - particle.velocity.y) * k,
                );
            }
        }
    }
    particle.velocity = Vec2::new(
        particle.velocity.x + acceleration.x * delta_seconds,
        particle.velocity.y + acceleration.y * delta_seconds,
    );
}

pub(crate) fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    let delta = target - current;
    if delta.abs() <= max_delta {
        target
    } else {
        current + delta.signum() * max_delta
    }
}

pub(crate) fn sample_spawn_area(area: ParticleSpawnArea2d, seed: &mut u64) -> Vec2 {
    match area {
        ParticleSpawnArea2d::Point => Vec2::ZERO,
        ParticleSpawnArea2d::Line { length } => {
            Vec2::new((next_unit(seed) - 0.5) * length.max(0.0), 0.0)
        }
        ParticleSpawnArea2d::Rect { size } => Vec2::new(
            (next_unit(seed) - 0.5) * size.x.max(0.0),
            (next_unit(seed) - 0.5) * size.y.max(0.0),
        ),
        ParticleSpawnArea2d::Circle { radius } => {
            let angle = next_unit(seed) * std::f32::consts::TAU;
            let radius = next_unit(seed).sqrt() * radius.max(0.0);
            Vec2::new(angle.cos() * radius, angle.sin() * radius)
        }
        ParticleSpawnArea2d::Ring {
            inner_radius,
            outer_radius,
        } => {
            let inner = inner_radius.max(0.0);
            let outer = outer_radius.max(inner);
            let angle = next_unit(seed) * std::f32::consts::TAU;
            let radius = inner + (outer - inner) * next_unit(seed);
            Vec2::new(angle.cos() * radius, angle.sin() * radius)
        }
    }
}

pub(crate) fn rotate_vec2(value: Vec2, radians: f32) -> Vec2 {
    let (sin, cos) = radians.sin_cos();
    Vec2::new(value.x * cos - value.y * sin, value.x * sin + value.y * cos)
}

pub(crate) fn local_to_world_position(position: Vec2, transform: Transform2) -> Vec2 {
    let rotated = rotate_vec2(position, transform.rotation_radians);
    Vec2::new(
        transform.translation.x + rotated.x,
        transform.translation.y + rotated.y,
    )
}

pub(crate) fn world_to_local_position(position: Vec2, transform: Transform2) -> Vec2 {
    rotate_vec2(
        Vec2::new(
            position.x - transform.translation.x,
            position.y - transform.translation.y,
        ),
        -transform.rotation_radians,
    )
}

pub(crate) fn hash_seed(value: &str) -> u64 {
    let mut hash = 1469598103934665603_u64;
    for byte in value.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

pub(crate) fn next_u32(seed: &mut u64) -> u32 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*seed >> 32) as u32
}

pub(crate) fn next_unit(seed: &mut u64) -> f32 {
    next_u32(seed) as f32 / u32::MAX as f32
}

pub(crate) fn next_signed_unit(seed: &mut u64) -> f32 {
    next_unit(seed) * 2.0 - 1.0
}
