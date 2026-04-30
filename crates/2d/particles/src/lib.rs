use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_fx::ColorRamp;
use amigo_math::{ColorRgba, Curve1d, Transform2, Vec2};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::SceneEntityId;

pub const PARTICLES_2D_PLUGIN_LABEL: &str = "amigo-2d-particles";
pub const PARTICLES_2D_CAPABILITY: &str = "particles_2d";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParticleShape2d {
    Circle { segments: u32 },
    Quad,
    Line { length: f32 },
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
    pub initial_size: f32,
    pub final_size: f32,
    pub color: ColorRgba,
    pub color_ramp: Option<ColorRamp>,
    pub z_index: f32,
    pub shape: ParticleShape2d,
    pub emission_rate_curve: Curve1d,
    pub size_curve: Curve1d,
    pub alpha_curve: Curve1d,
    pub speed_curve: Curve1d,
    pub forces: Vec<ParticleForce2d>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleEmitter2dCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub emitter: ParticleEmitter2d,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle2d {
    pub position: Vec2,
    pub velocity: Vec2,
    pub age: f32,
    pub lifetime: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Particle2dDrawCommand {
    pub emitter_entity_name: String,
    pub position: Vec2,
    pub size: f32,
    pub color: ColorRgba,
    pub z_index: f32,
    pub shape: ParticleShape2d,
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

#[derive(Debug, Default)]
pub struct Particle2dSceneService {
    state: Mutex<Particle2dState>,
}

#[derive(Debug, Default)]
struct Particle2dState {
    emitters: BTreeMap<String, ParticleEmitter2dCommand>,
    particles: BTreeMap<String, Vec<Particle2d>>,
    emission_accumulators: BTreeMap<String, f32>,
    active_overrides: BTreeMap<String, bool>,
    intensities: BTreeMap<String, f32>,
    rng_states: BTreeMap<String, u64>,
    pending_bursts: BTreeMap<String, usize>,
}

impl Particle2dSceneService {
    pub fn queue_emitter(&self, command: ParticleEmitter2dCommand) {
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        state
            .emitters
            .insert(command.entity_name.clone(), command.clone());
        state
            .particles
            .entry(command.entity_name.clone())
            .or_default();
        state
            .emission_accumulators
            .entry(command.entity_name.clone())
            .or_insert(0.0);
        state
            .rng_states
            .entry(command.entity_name.clone())
            .or_insert_with(|| hash_seed(command.entity_name.as_str()));
    }

    pub fn clear(&self) {
        *self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned") =
            Particle2dState::default();
    }

    pub fn emitter(&self, entity_name: &str) -> Option<ParticleEmitter2dCommand> {
        self.state
            .lock()
            .expect("particle scene service mutex should not be poisoned")
            .emitters
            .get(entity_name)
            .cloned()
    }

    pub fn emitters(&self) -> Vec<ParticleEmitter2dCommand> {
        self.state
            .lock()
            .expect("particle scene service mutex should not be poisoned")
            .emitters
            .values()
            .cloned()
            .collect()
    }

    pub fn set_active(&self, entity_name: &str, active: bool) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        if !state.emitters.contains_key(entity_name) {
            return false;
        }
        state
            .active_overrides
            .insert(entity_name.to_owned(), active);
        true
    }

    pub fn is_active(&self, entity_name: &str) -> bool {
        let state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        state
            .active_overrides
            .get(entity_name)
            .copied()
            .or_else(|| {
                state
                    .emitters
                    .get(entity_name)
                    .map(|command| command.emitter.active)
            })
            .unwrap_or(false)
    }

    pub fn set_intensity(&self, entity_name: &str, intensity: f32) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        if !state.emitters.contains_key(entity_name) || !intensity.is_finite() {
            return false;
        }
        state
            .intensities
            .insert(entity_name.to_owned(), intensity.clamp(0.0, 1.0));
        true
    }

    pub fn set_spawn_rate(&self, entity_name: &str, spawn_rate: f32) -> bool {
        if !spawn_rate.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.spawn_rate = spawn_rate.max(0.0);
        })
    }

    pub fn set_particle_lifetime(&self, entity_name: &str, lifetime: f32) -> bool {
        if !lifetime.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.particle_lifetime = lifetime.max(0.001);
        })
    }

    pub fn set_max_particles(&self, entity_name: &str, max_particles: usize) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.max_particles = max_particles;
        })
    }

    pub fn set_initial_speed(&self, entity_name: &str, speed: f32) -> bool {
        if !speed.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.initial_speed = speed.max(0.0);
        })
    }

    pub fn set_spread_radians(&self, entity_name: &str, spread_radians: f32) -> bool {
        if !spread_radians.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.spread_radians = spread_radians.max(0.0);
        })
    }

    pub fn set_initial_size(&self, entity_name: &str, size: f32) -> bool {
        if !size.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.initial_size = size.max(0.0);
        })
    }

    pub fn set_final_size(&self, entity_name: &str, size: f32) -> bool {
        if !size.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.final_size = size.max(0.0);
        })
    }

    pub fn set_color(&self, entity_name: &str, color: ColorRgba) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.color = color;
            emitter.color_ramp = None;
        })
    }

    pub fn set_color_ramp(&self, entity_name: &str, color_ramp: ColorRamp) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.color_ramp = Some(color_ramp);
        })
    }

    pub fn clear_color_ramp(&self, entity_name: &str) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.color_ramp = None;
        })
    }

    pub fn set_gravity(&self, entity_name: &str, x: f32, y: f32) -> bool {
        if !x.is_finite() || !y.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter
                .forces
                .retain(|force| !matches!(force, ParticleForce2d::Gravity { .. }));
            emitter.forces.push(ParticleForce2d::Gravity {
                acceleration: Vec2::new(x, y),
            });
        })
    }

    pub fn set_drag(&self, entity_name: &str, coefficient: f32) -> bool {
        if !coefficient.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter
                .forces
                .retain(|force| !matches!(force, ParticleForce2d::Drag { .. }));
            emitter.forces.push(ParticleForce2d::Drag {
                coefficient: coefficient.max(0.0),
            });
        })
    }

    pub fn set_wind(&self, entity_name: &str, x: f32, y: f32, strength: f32) -> bool {
        if !x.is_finite() || !y.is_finite() || !strength.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter
                .forces
                .retain(|force| !matches!(force, ParticleForce2d::Wind { .. }));
            emitter.forces.push(ParticleForce2d::Wind {
                velocity: Vec2::new(x, y),
                strength: strength.max(0.0),
            });
        })
    }

    pub fn clear_forces(&self, entity_name: &str) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.forces.clear();
        })
    }

    pub fn set_spawn_area(&self, entity_name: &str, spawn_area: ParticleSpawnArea2d) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.spawn_area = spawn_area;
        })
    }

    pub fn set_shape(&self, entity_name: &str, shape: ParticleShape2d) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.shape = shape;
        })
    }

    pub fn burst(&self, entity_name: &str, count: usize) -> bool {
        if count == 0 {
            return true;
        }
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        if !state.emitters.contains_key(entity_name) {
            return false;
        }
        *state
            .pending_bursts
            .entry(entity_name.to_owned())
            .or_default() += count;
        true
    }

    fn update_emitter(
        &self,
        entity_name: &str,
        update: impl FnOnce(&mut ParticleEmitter2d),
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        let Some(command) = state.emitters.get_mut(entity_name) else {
            return false;
        };
        update(&mut command.emitter);
        true
    }

    pub fn intensity(&self, entity_name: &str) -> f32 {
        self.state
            .lock()
            .expect("particle scene service mutex should not be poisoned")
            .intensities
            .get(entity_name)
            .copied()
            .unwrap_or(1.0)
    }

    pub fn particle_count(&self, entity_name: &str) -> usize {
        self.state
            .lock()
            .expect("particle scene service mutex should not be poisoned")
            .particles
            .get(entity_name)
            .map(Vec::len)
            .unwrap_or(0)
    }

    pub fn tick(&self, inputs: &[Particle2dEmitterRuntimeInput], delta_seconds: f32) {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return;
        }
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        let input_lookup = inputs
            .iter()
            .map(|input| (input.emitter_entity_name.as_str(), input))
            .collect::<BTreeMap<_, _>>();
        let emitters = state.emitters.values().cloned().collect::<Vec<_>>();

        for command in emitters {
            let entity_name = command.entity_name.as_str();
            let emitter = command.emitter;
            let particles = state
                .particles
                .entry(command.entity_name.clone())
                .or_default();
            for particle in particles.iter_mut() {
                particle.age = (particle.age + delta_seconds).min(particle.lifetime);
                apply_particle_forces(particle, &emitter.forces, delta_seconds);
                particle.position = Vec2::new(
                    particle.position.x + particle.velocity.x * delta_seconds,
                    particle.position.y + particle.velocity.y * delta_seconds,
                );
            }
            particles.retain(|particle| particle.age < particle.lifetime);

            let active = state
                .active_overrides
                .get(entity_name)
                .copied()
                .unwrap_or(emitter.active);
            let Some(input) = input_lookup.get(entity_name).copied() else {
                continue;
            };
            if !input.source_visible || !input.source_simulation_enabled {
                continue;
            }

            let intensity = state
                .intensities
                .get(entity_name)
                .copied()
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);
            if emitter.max_particles == 0 {
                continue;
            }

            let mut spawn_count = 0usize;
            if active {
                let rate =
                    emitter.spawn_rate.max(0.0) * emitter.emission_rate_curve.sample(intensity);
                if rate > 0.0 {
                    let accumulator = state
                        .emission_accumulators
                        .entry(entity_name.to_owned())
                        .or_insert(0.0);
                    *accumulator += rate * delta_seconds;
                    spawn_count = accumulator.floor() as usize;
                    *accumulator -= spawn_count as f32;
                }
            }
            spawn_count = spawn_count
                .saturating_add(state.pending_bursts.remove(entity_name).unwrap_or_default());
            if spawn_count == 0 {
                continue;
            }

            let live_count = state
                .particles
                .get(entity_name)
                .map(Vec::len)
                .unwrap_or_default();
            spawn_count = spawn_count.min(emitter.max_particles.saturating_sub(live_count));

            for _ in 0..spawn_count {
                let seed = state
                    .rng_states
                    .entry(entity_name.to_owned())
                    .or_insert_with(|| hash_seed(entity_name));
                let particle = spawn_particle(&emitter, input, intensity, seed);
                state
                    .particles
                    .entry(entity_name.to_owned())
                    .or_default()
                    .push(particle);
            }
        }
    }

    pub fn draw_commands(&self) -> Vec<Particle2dDrawCommand> {
        let state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        let mut commands = Vec::new();
        for (entity_name, particles) in &state.particles {
            let Some(emitter) = state
                .emitters
                .get(entity_name)
                .map(|command| &command.emitter)
            else {
                continue;
            };
            for particle in particles {
                let age_t = if particle.lifetime <= f32::EPSILON {
                    1.0
                } else {
                    (particle.age / particle.lifetime).clamp(0.0, 1.0)
                };
                let size_t = emitter.size_curve.sample(age_t);
                let size =
                    emitter.initial_size + (emitter.final_size - emitter.initial_size) * size_t;
                let alpha = emitter.alpha_curve.sample(age_t).clamp(0.0, 1.0);
                let sampled_color = emitter
                    .color_ramp
                    .as_ref()
                    .map(|ramp| ramp.sample(age_t))
                    .unwrap_or(emitter.color);
                commands.push(Particle2dDrawCommand {
                    emitter_entity_name: entity_name.clone(),
                    position: particle.position,
                    size,
                    color: ColorRgba::new(
                        sampled_color.r,
                        sampled_color.g,
                        sampled_color.b,
                        sampled_color.a * alpha,
                    ),
                    z_index: emitter.z_index,
                    shape: emitter.shape,
                    transform: Transform2 {
                        rotation_radians: particle.velocity.y.atan2(particle.velocity.x),
                        ..Transform2::default()
                    },
                });
            }
        }
        commands
    }
}

fn spawn_particle(
    emitter: &ParticleEmitter2d,
    input: &Particle2dEmitterRuntimeInput,
    intensity: f32,
    seed: &mut u64,
) -> Particle2d {
    let parent_rotation = input.source_transform.rotation_radians;
    let emitter_rotation = parent_rotation + emitter.local_direction_radians;
    let offset = rotate_vec2(emitter.local_offset, parent_rotation);
    let area_offset = rotate_vec2(sample_spawn_area(emitter.spawn_area, seed), parent_rotation);
    let position = Vec2::new(
        input.source_transform.translation.x + offset.x + area_offset.x,
        input.source_transform.translation.y + offset.y + area_offset.y,
    );
    let spread = next_signed_unit(seed) * emitter.spread_radians * 0.5;
    let direction_angle = emitter_rotation + spread;
    let speed_jitter = next_signed_unit(seed) * emitter.speed_jitter.max(0.0);
    let speed =
        (emitter.initial_speed * emitter.speed_curve.sample(intensity) + speed_jitter).max(0.0);
    let lifetime_jitter = next_signed_unit(seed) * emitter.lifetime_jitter.max(0.0);
    let lifetime = (emitter.particle_lifetime + lifetime_jitter).max(0.001);
    let direction = Vec2::new(direction_angle.cos(), direction_angle.sin());
    Particle2d {
        position,
        velocity: Vec2::new(
            direction.x * speed + input.source_velocity.x * emitter.inherit_parent_velocity,
            direction.y * speed + input.source_velocity.y * emitter.inherit_parent_velocity,
        ),
        age: 0.0,
        lifetime,
    }
}

fn apply_particle_forces(
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

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    let delta = target - current;
    if delta.abs() <= max_delta {
        target
    } else {
        current + delta.signum() * max_delta
    }
}

fn sample_spawn_area(area: ParticleSpawnArea2d, seed: &mut u64) -> Vec2 {
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

fn rotate_vec2(value: Vec2, radians: f32) -> Vec2 {
    let (sin, cos) = radians.sin_cos();
    Vec2::new(value.x * cos - value.y * sin, value.x * sin + value.y * cos)
}

fn hash_seed(value: &str) -> u64 {
    let mut hash = 1469598103934665603_u64;
    for byte in value.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

fn next_u32(seed: &mut u64) -> u32 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*seed >> 32) as u32
}

fn next_unit(seed: &mut u64) -> f32 {
    next_u32(seed) as f32 / u32::MAX as f32
}

fn next_signed_unit(seed: &mut u64) -> f32 {
    next_unit(seed) * 2.0 - 1.0
}

#[derive(Debug, Clone)]
pub struct Particle2dDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct Particle2dPlugin;

impl RuntimePlugin for Particle2dPlugin {
    fn name(&self) -> &'static str {
        PARTICLES_2D_PLUGIN_LABEL
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(Particle2dSceneService::default())?;
        registry.register(Particle2dDomainInfo {
            crate_name: "amigo-2d-particles",
            capability: PARTICLES_2D_CAPABILITY,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_emitter(active: bool) -> ParticleEmitter2dCommand {
        ParticleEmitter2dCommand {
            entity_id: SceneEntityId::new(1),
            entity_name: "thruster".to_owned(),
            emitter: ParticleEmitter2d {
                attached_to: Some("ship".to_owned()),
                local_offset: Vec2::ZERO,
                local_direction_radians: 0.0,
                spawn_area: ParticleSpawnArea2d::Point,
                active,
                spawn_rate: 10.0,
                max_particles: 10,
                particle_lifetime: 1.0,
                lifetime_jitter: 0.0,
                initial_speed: 10.0,
                speed_jitter: 0.0,
                spread_radians: 0.0,
                inherit_parent_velocity: 0.0,
                initial_size: 2.0,
                final_size: 6.0,
                color: ColorRgba::WHITE,
                color_ramp: None,
                z_index: 1.0,
                shape: ParticleShape2d::Circle { segments: 8 },
                emission_rate_curve: Curve1d::Constant(1.0),
                size_curve: Curve1d::Linear,
                alpha_curve: Curve1d::Constant(1.0),
                speed_curve: Curve1d::Constant(1.0),
                forces: Vec::new(),
            },
        }
    }

    fn test_input() -> Particle2dEmitterRuntimeInput {
        Particle2dEmitterRuntimeInput {
            emitter_entity_name: "thruster".to_owned(),
            source_entity_name: "ship".to_owned(),
            source_transform: Transform2::default(),
            source_velocity: Vec2::ZERO,
            source_visible: true,
            source_simulation_enabled: true,
        }
    }

    #[test]
    fn active_emitter_spawns_particles() {
        let service = Particle2dSceneService::default();
        service.queue_emitter(test_emitter(true));

        service.tick(&[test_input()], 0.2);

        assert_eq!(service.particle_count("thruster"), 2);
    }

    #[test]
    fn inactive_emitter_does_not_spawn() {
        let service = Particle2dSceneService::default();
        service.queue_emitter(test_emitter(false));

        service.tick(&[test_input()], 0.5);

        assert_eq!(service.particle_count("thruster"), 0);
    }

    #[test]
    fn existing_particles_age_and_expire() {
        let service = Particle2dSceneService::default();
        service.queue_emitter(test_emitter(true));
        service.tick(&[test_input()], 0.2);
        service.set_active("thruster", false);

        service.tick(&[test_input()], 1.0);

        assert_eq!(service.particle_count("thruster"), 0);
    }

    #[test]
    fn max_particles_caps_runtime_particles() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.max_particles = 3;
        command.emitter.spawn_rate = 100.0;
        service.queue_emitter(command);

        service.tick(&[test_input()], 1.0);

        assert_eq!(service.particle_count("thruster"), 3);
    }

    #[test]
    fn size_curve_grows_particle_over_lifetime() {
        let service = Particle2dSceneService::default();
        service.queue_emitter(test_emitter(true));
        service.tick(&[test_input()], 0.1);
        service.set_active("thruster", false);
        service.tick(&[test_input()], 0.4);

        let draw = service.draw_commands();

        assert!(draw[0].size > 2.0);
    }

    #[test]
    fn alpha_curve_fades_particle_over_lifetime() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.alpha_curve = Curve1d::Custom {
            points: vec![
                amigo_math::CurvePoint1d { t: 0.0, value: 1.0 },
                amigo_math::CurvePoint1d { t: 1.0, value: 0.0 },
            ],
        };
        service.queue_emitter(command);
        service.tick(&[test_input()], 0.1);
        service.set_active("thruster", false);
        service.tick(&[test_input()], 0.4);

        let draw = service.draw_commands();

        assert!(draw[0].color.a < 1.0);
    }

    #[test]
    fn particle_color_ramp_changes_rgb_over_lifetime() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.color = ColorRgba::new(1.0, 1.0, 1.0, 1.0);
        command.emitter.color_ramp = Some(ColorRamp {
            interpolation: amigo_fx::ColorInterpolation::LinearRgb,
            stops: vec![
                amigo_fx::ColorStop {
                    t: 0.0,
                    color: ColorRgba::new(1.0, 0.0, 0.0, 1.0),
                },
                amigo_fx::ColorStop {
                    t: 1.0,
                    color: ColorRgba::new(0.0, 0.0, 1.0, 1.0),
                },
            ],
        });
        service.queue_emitter(command);
        service.tick(&[test_input()], 0.1);
        service.set_active("thruster", false);
        service.tick(&[test_input()], 0.4);

        let color = service.draw_commands()[0].color;

        assert!(color.r < 1.0);
        assert!(color.b > 0.0);
    }

    #[test]
    fn particle_color_ramp_alpha_multiplies_alpha_curve() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.color_ramp = Some(ColorRamp::constant(ColorRgba::new(1.0, 0.0, 0.0, 0.5)));
        command.emitter.alpha_curve = Curve1d::Constant(0.5);
        service.queue_emitter(command);
        service.tick(&[test_input()], 0.1);

        let color = service.draw_commands()[0].color;

        assert!((color.a - 0.25).abs() < 0.001);
    }

    #[test]
    fn particle_missing_color_ramp_preserves_legacy_color() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.color = ColorRgba::new(0.25, 0.5, 0.75, 1.0);
        service.queue_emitter(command);
        service.tick(&[test_input()], 0.1);

        let color = service.draw_commands()[0].color;

        assert_eq!(color, ColorRgba::new(0.25, 0.5, 0.75, 1.0));
    }

    #[test]
    fn set_color_clears_color_ramp_override() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.color_ramp = Some(ColorRamp::constant(ColorRgba::new(1.0, 0.0, 0.0, 1.0)));
        service.queue_emitter(command);

        assert!(service.set_color("thruster", ColorRgba::new(0.0, 1.0, 0.0, 1.0)));

        let emitter = service.emitter("thruster").expect("emitter should exist");
        assert_eq!(emitter.emitter.color, ColorRgba::new(0.0, 1.0, 0.0, 1.0));
        assert!(emitter.emitter.color_ramp.is_none());
    }

    #[test]
    fn set_color_ramp_updates_draw_color() {
        let service = Particle2dSceneService::default();
        service.queue_emitter(test_emitter(true));

        assert!(service.set_color_ramp(
            "thruster",
            ColorRamp::constant(ColorRgba::new(0.0, 1.0, 0.0, 1.0))
        ));
        service.tick(&[test_input()], 0.1);

        assert_eq!(
            service.draw_commands()[0].color,
            ColorRgba::new(0.0, 1.0, 0.0, 1.0)
        );
    }

    #[test]
    fn clear_color_ramp_restores_legacy_color() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.color = ColorRgba::new(0.2, 0.3, 0.4, 1.0);
        command.emitter.color_ramp = Some(ColorRamp::constant(ColorRgba::new(1.0, 0.0, 0.0, 1.0)));
        service.queue_emitter(command);

        assert!(service.clear_color_ramp("thruster"));
        service.tick(&[test_input()], 0.1);

        assert_eq!(
            service.draw_commands()[0].color,
            ColorRgba::new(0.2, 0.3, 0.4, 1.0)
        );
    }

    #[test]
    fn emission_rate_curve_modulates_spawn_count() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.emission_rate_curve = Curve1d::Constant(0.5);
        service.queue_emitter(command);

        service.tick(&[test_input()], 1.0);

        assert_eq!(service.particle_count("thruster"), 5);
    }

    #[test]
    fn parent_velocity_is_inherited() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.inherit_parent_velocity = 0.5;
        command.emitter.initial_speed = 0.0;
        service.queue_emitter(command);
        let mut input = test_input();
        input.source_velocity = Vec2::new(20.0, 0.0);

        service.tick(&[input], 0.1);
        let first = service.draw_commands().remove(0);

        assert!(first.position.x >= 0.0);
    }

    #[test]
    fn burst_spawns_particles_when_emitter_inactive() {
        let service = Particle2dSceneService::default();
        service.queue_emitter(test_emitter(false));

        assert!(service.burst("thruster", 4));
        service.tick(&[test_input()], 0.1);

        assert_eq!(service.particle_count("thruster"), 4);
    }

    #[test]
    fn burst_respects_max_particles() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(false);
        command.emitter.max_particles = 3;
        service.queue_emitter(command);

        assert!(service.burst("thruster", 8));
        service.tick(&[test_input()], 0.1);

        assert_eq!(service.particle_count("thruster"), 3);
    }

    #[test]
    fn burst_missing_emitter_returns_false() {
        let service = Particle2dSceneService::default();

        assert!(!service.burst("missing", 1));
    }

    #[test]
    fn gravity_changes_particle_velocity() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.initial_speed = 0.0;
        command.emitter.forces = vec![ParticleForce2d::Gravity {
            acceleration: Vec2::new(0.0, -10.0),
        }];
        service.queue_emitter(command);
        service.tick(&[test_input()], 0.1);
        service.set_active("thruster", false);

        service.tick(&[test_input()], 0.1);

        let draw = service.draw_commands();
        assert!(draw[0].position.y < 0.0);
    }

    #[test]
    fn drag_reduces_particle_velocity() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.forces = vec![ParticleForce2d::Drag { coefficient: 5.0 }];
        service.queue_emitter(command);
        service.tick(&[test_input()], 0.1);
        service.set_active("thruster", false);

        service.tick(&[test_input()], 0.1);

        let draw = service.draw_commands();
        assert!(draw[0].position.x < 2.0);
    }

    #[test]
    fn wind_moves_velocity_toward_wind_velocity() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.initial_speed = 0.0;
        command.emitter.forces = vec![ParticleForce2d::Wind {
            velocity: Vec2::new(20.0, 0.0),
            strength: 10.0,
        }];
        service.queue_emitter(command);
        service.tick(&[test_input()], 0.1);
        service.set_active("thruster", false);

        service.tick(&[test_input()], 0.1);

        let draw = service.draw_commands();
        assert!(draw[0].position.x > 0.0);
    }

    #[test]
    fn rect_spawn_area_offsets_particles_within_bounds() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.spawn_area = ParticleSpawnArea2d::Rect {
            size: Vec2::new(10.0, 20.0),
        };
        command.emitter.initial_speed = 0.0;
        service.queue_emitter(command);

        service.tick(&[test_input()], 0.1);

        let position = service.draw_commands()[0].position;
        assert!(position.x.abs() <= 5.0);
        assert!(position.y.abs() <= 10.0);
    }

    #[test]
    fn circle_spawn_area_offsets_particles_within_radius() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.spawn_area = ParticleSpawnArea2d::Circle { radius: 12.0 };
        command.emitter.initial_speed = 0.0;
        service.queue_emitter(command);

        service.tick(&[test_input()], 0.1);

        let position = service.draw_commands()[0].position;
        assert!((position.x * position.x + position.y * position.y).sqrt() <= 12.0);
    }

    #[test]
    fn set_max_particles_caps_future_particles() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.spawn_rate = 100.0;
        service.queue_emitter(command);

        assert!(service.set_max_particles("thruster", 2));
        service.tick(&[test_input()], 1.0);

        assert_eq!(service.particle_count("thruster"), 2);
    }

    #[test]
    fn set_wind_replaces_existing_wind_force() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.forces = vec![ParticleForce2d::Wind {
            velocity: Vec2::new(1.0, 0.0),
            strength: 1.0,
        }];
        service.queue_emitter(command);

        assert!(service.set_wind("thruster", 20.0, 5.0, 2.0));

        let emitter = service.emitter("thruster").expect("emitter should exist");
        assert_eq!(emitter.emitter.forces.len(), 1);
        assert!(matches!(
            emitter.emitter.forces[0],
            ParticleForce2d::Wind {
                velocity,
                strength
            } if velocity == Vec2::new(20.0, 5.0) && (strength - 2.0).abs() < 0.001
        ));
    }

    #[test]
    fn line_spawn_area_offsets_particles_within_length() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.spawn_area = ParticleSpawnArea2d::Line { length: 20.0 };
        command.emitter.initial_speed = 0.0;
        service.queue_emitter(command);

        service.tick(&[test_input()], 0.1);

        let position = service.draw_commands()[0].position;
        assert!(position.x.abs() <= 10.0);
    }

    #[test]
    fn ring_spawn_area_offsets_particles_between_radii() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.spawn_area = ParticleSpawnArea2d::Ring {
            inner_radius: 8.0,
            outer_radius: 16.0,
        };
        command.emitter.initial_speed = 0.0;
        service.queue_emitter(command);

        service.tick(&[test_input()], 0.1);

        let position = service.draw_commands()[0].position;
        let radius = (position.x * position.x + position.y * position.y).sqrt();
        assert!((8.0..=16.0).contains(&radius));
    }

    #[test]
    fn set_shape_changes_draw_command_shape() {
        let service = Particle2dSceneService::default();
        service.queue_emitter(test_emitter(true));

        assert!(service.set_shape("thruster", ParticleShape2d::Line { length: 12.0 }));
        service.tick(&[test_input()], 0.1);

        assert_eq!(
            service.draw_commands()[0].shape,
            ParticleShape2d::Line { length: 12.0 }
        );
    }
}
