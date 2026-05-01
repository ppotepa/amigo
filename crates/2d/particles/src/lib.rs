use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_fx::ColorRamp;
use amigo_math::{ColorRgba, Curve1d, Transform2, Vec2};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{
    ParticleAlignMode2dSceneCommand, ParticleEmitter2dSceneCommand, ParticleForce2dSceneCommand,
    ParticleShape2dSceneCommand, ParticleSpawnArea2dSceneCommand, SceneEntityId,
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
pub enum ParticleAlignMode2d {
    None,
    Velocity,
    Emitter,
    Random,
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
    pub align: ParticleAlignMode2d,
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
            initial_size: command.initial_size,
            final_size: command.final_size,
            color: command.color,
            color_ramp: command.color_ramp.clone(),
            z_index: command.z_index,
            shape: particle_shape_from_scene_command(command.shape),
            align: particle_align_from_scene_command(command.align),
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
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation_radians: f32,
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
pub struct ParticlePreset2dService {
    presets: Mutex<BTreeMap<String, ParticlePreset2d>>,
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
    pending_positioned_bursts: BTreeMap<String, Vec<PositionedParticleBurst2d>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PositionedParticleBurst2d {
    position: Vec2,
    count: usize,
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

    pub fn emitter_yaml(&self, entity_name: &str) -> Option<String> {
        self.emitter(entity_name)
            .map(|command| particle_emitter_to_scene_yaml(&command.emitter))
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

    pub fn set_lifetime_jitter(&self, entity_name: &str, jitter: f32) -> bool {
        if !jitter.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.lifetime_jitter = jitter.max(0.0);
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

    pub fn set_speed_jitter(&self, entity_name: &str, jitter: f32) -> bool {
        if !jitter.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.speed_jitter = jitter.max(0.0);
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

    pub fn set_local_direction_radians(&self, entity_name: &str, radians: f32) -> bool {
        if !radians.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.local_direction_radians = radians;
        })
    }

    pub fn set_inherit_parent_velocity(&self, entity_name: &str, scale: f32) -> bool {
        if !scale.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.inherit_parent_velocity = scale;
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

    pub fn set_align(&self, entity_name: &str, align: ParticleAlignMode2d) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.align = align;
        })
    }

    pub fn copy_emitter_config(&self, source_entity_name: &str, target_entity_name: &str) -> bool {
        let Some(source) = self.emitter(source_entity_name) else {
            return false;
        };
        self.replace_emitter_config(target_entity_name, source.emitter)
    }

    pub fn replace_emitter_config(
        &self,
        target_entity_name: &str,
        emitter: ParticleEmitter2d,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        let Some(target) = state.emitters.get_mut(target_entity_name) else {
            return false;
        };

        target.emitter = emitter;
        state
            .particles
            .entry(target_entity_name.to_owned())
            .or_default()
            .clear();
        state
            .emission_accumulators
            .insert(target_entity_name.to_owned(), 0.0);
        state.active_overrides.remove(target_entity_name);
        state.intensities.remove(target_entity_name);
        true
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

    pub fn burst_at(&self, entity_name: &str, position: Vec2, count: usize) -> bool {
        if count == 0 {
            return true;
        }
        if !position.x.is_finite() || !position.y.is_finite() {
            return false;
        }
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        if !state.emitters.contains_key(entity_name) {
            return false;
        }
        state
            .pending_positioned_bursts
            .entry(entity_name.to_owned())
            .or_default()
            .push(PositionedParticleBurst2d { position, count });
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
            let positioned_bursts = state
                .pending_positioned_bursts
                .remove(entity_name)
                .unwrap_or_default();
            if spawn_count == 0 && positioned_bursts.is_empty() {
                continue;
            }

            let mut live_count = state
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
                live_count += 1;
            }

            for burst in positioned_bursts {
                let remaining = emitter.max_particles.saturating_sub(live_count);
                let positioned_count = burst.count.min(remaining);
                if positioned_count == 0 {
                    break;
                }
                for _ in 0..positioned_count {
                    let seed = state
                        .rng_states
                        .entry(entity_name.to_owned())
                        .or_insert_with(|| hash_seed(entity_name));
                    let particle =
                        spawn_particle_at(&emitter, input, intensity, seed, Some(burst.position));
                    state
                        .particles
                        .entry(entity_name.to_owned())
                        .or_default()
                        .push(particle);
                    live_count += 1;
                }
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
                        rotation_radians: particle_rotation_for_align(particle, emitter.align),
                        ..Transform2::default()
                    },
                });
            }
        }
        commands
    }
}

impl ParticlePreset2dService {
    pub fn register(&self, preset: ParticlePreset2d) -> bool {
        if preset.id.is_empty() {
            return false;
        }
        self.presets
            .lock()
            .expect("particle preset service mutex should not be poisoned")
            .insert(preset.id.clone(), preset)
            .is_none()
    }

    pub fn clear(&self) {
        self.presets
            .lock()
            .expect("particle preset service mutex should not be poisoned")
            .clear();
    }

    pub fn ids(&self) -> Vec<String> {
        self.presets
            .lock()
            .expect("particle preset service mutex should not be poisoned")
            .keys()
            .cloned()
            .collect()
    }

    pub fn preset(&self, id: &str) -> Option<ParticlePreset2d> {
        self.presets
            .lock()
            .expect("particle preset service mutex should not be poisoned")
            .get(id)
            .cloned()
    }

    pub fn len(&self) -> usize {
        self.presets
            .lock()
            .expect("particle preset service mutex should not be poisoned")
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn apply_to_emitter(
        &self,
        particle_scene: &Particle2dSceneService,
        preset_id: &str,
        target_entity_name: &str,
    ) -> bool {
        let Some(preset) = self.preset(preset_id) else {
            return false;
        };
        particle_scene.replace_emitter_config(target_entity_name, preset.emitter)
    }
}

fn spawn_particle(
    emitter: &ParticleEmitter2d,
    input: &Particle2dEmitterRuntimeInput,
    intensity: f32,
    seed: &mut u64,
) -> Particle2d {
    spawn_particle_at(emitter, input, intensity, seed, None)
}

fn spawn_particle_at(
    emitter: &ParticleEmitter2d,
    input: &Particle2dEmitterRuntimeInput,
    intensity: f32,
    seed: &mut u64,
    position_override: Option<Vec2>,
) -> Particle2d {
    let parent_rotation = input.source_transform.rotation_radians;
    let emitter_rotation = parent_rotation + emitter.local_direction_radians;
    let offset = rotate_vec2(emitter.local_offset, parent_rotation);
    let area_offset = rotate_vec2(sample_spawn_area(emitter.spawn_area, seed), parent_rotation);
    let position = position_override.unwrap_or_else(|| {
        Vec2::new(
            input.source_transform.translation.x + offset.x + area_offset.x,
            input.source_transform.translation.y + offset.y + area_offset.y,
        )
    });
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
    Particle2d {
        position,
        velocity: Vec2::new(
            direction.x * speed + input.source_velocity.x * emitter.inherit_parent_velocity,
            direction.y * speed + input.source_velocity.y * emitter.inherit_parent_velocity,
        ),
        rotation_radians,
        age: 0.0,
        lifetime,
    }
}

fn particle_rotation_for_align(particle: &Particle2d, align: ParticleAlignMode2d) -> f32 {
    match align {
        ParticleAlignMode2d::None => 0.0,
        ParticleAlignMode2d::Velocity => particle.velocity.y.atan2(particle.velocity.x),
        ParticleAlignMode2d::Emitter | ParticleAlignMode2d::Random => particle.rotation_radians,
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

pub fn particle_emitter_to_scene_yaml(emitter: &ParticleEmitter2d) -> String {
    let mut yaml = String::new();
    yaml.push_str("type: ParticleEmitter2D\n");
    if let Some(attached_to) = emitter.attached_to.as_deref() {
        yaml.push_str(&format!("attached_to: {attached_to}\n"));
    }
    yaml.push_str(&format!(
        "local_offset: {{ x: {}, y: {} }}\n",
        fmt_f32(emitter.local_offset.x),
        fmt_f32(emitter.local_offset.y)
    ));
    yaml.push_str(&format!(
        "local_direction_degrees: {}\n",
        fmt_f32(emitter.local_direction_radians.to_degrees())
    ));
    yaml.push_str(&format!("active: {}\n", emitter.active));
    yaml.push_str(&format!("spawn_rate: {}\n", fmt_f32(emitter.spawn_rate)));
    yaml.push_str(&format!("max_particles: {}\n", emitter.max_particles));
    yaml.push_str(&format!(
        "particle_lifetime: {}\n",
        fmt_f32(emitter.particle_lifetime)
    ));
    yaml.push_str(&format!(
        "lifetime_jitter: {}\n",
        fmt_f32(emitter.lifetime_jitter)
    ));
    yaml.push_str(&format!(
        "initial_speed: {}\n",
        fmt_f32(emitter.initial_speed)
    ));
    yaml.push_str(&format!(
        "speed_jitter: {}\n",
        fmt_f32(emitter.speed_jitter)
    ));
    yaml.push_str(&format!(
        "spread_degrees: {}\n",
        fmt_f32(emitter.spread_radians.to_degrees())
    ));
    yaml.push_str(&format!(
        "inherit_parent_velocity: {}\n",
        fmt_f32(emitter.inherit_parent_velocity)
    ));
    yaml.push_str(&format!(
        "initial_size: {}\n",
        fmt_f32(emitter.initial_size)
    ));
    yaml.push_str(&format!("final_size: {}\n", fmt_f32(emitter.final_size)));
    yaml.push_str(&format!("color: \"{}\"\n", color_to_hex(emitter.color)));
    if let Some(color_ramp) = emitter.color_ramp.as_ref() {
        yaml.push_str("color_ramp:\n");
        yaml.push_str(&format!(
            "  interpolation: {}\n",
            color_interpolation_name(color_ramp.interpolation)
        ));
        yaml.push_str("  stops:\n");
        for stop in &color_ramp.stops {
            yaml.push_str(&format!(
                "    - {{ t: {}, color: \"{}\" }}\n",
                fmt_f32(stop.t),
                color_to_hex(stop.color)
            ));
        }
    }
    yaml.push_str("spawn_area:\n");
    append_spawn_area_yaml(&mut yaml, emitter.spawn_area, "  ");
    yaml.push_str("shape:\n");
    append_shape_yaml(&mut yaml, emitter.shape, "  ");
    yaml.push_str(&format!("align: {}\n", align_name(emitter.align)));
    if !emitter.forces.is_empty() {
        yaml.push_str("forces:\n");
        for force in &emitter.forces {
            append_force_yaml(&mut yaml, *force, "  ");
        }
    }
    yaml.push_str(&format!(
        "emission_rate_curve: {}\n",
        inline_curve_yaml(&emitter.emission_rate_curve)
    ));
    yaml.push_str(&format!(
        "size_curve: {}\n",
        inline_curve_yaml(&emitter.size_curve)
    ));
    yaml.push_str(&format!(
        "alpha_curve: {}\n",
        inline_curve_yaml(&emitter.alpha_curve)
    ));
    yaml.push_str(&format!(
        "speed_curve: {}\n",
        inline_curve_yaml(&emitter.speed_curve)
    ));
    yaml
}

fn append_spawn_area_yaml(yaml: &mut String, spawn_area: ParticleSpawnArea2d, indent: &str) {
    match spawn_area {
        ParticleSpawnArea2d::Point => yaml.push_str(&format!("{indent}kind: point\n")),
        ParticleSpawnArea2d::Line { length } => yaml.push_str(&format!(
            "{indent}kind: line\n{indent}length: {}\n",
            fmt_f32(length)
        )),
        ParticleSpawnArea2d::Rect { size } => yaml.push_str(&format!(
            "{indent}kind: rect\n{indent}size: {{ x: {}, y: {} }}\n",
            fmt_f32(size.x),
            fmt_f32(size.y)
        )),
        ParticleSpawnArea2d::Circle { radius } => yaml.push_str(&format!(
            "{indent}kind: circle\n{indent}radius: {}\n",
            fmt_f32(radius)
        )),
        ParticleSpawnArea2d::Ring {
            inner_radius,
            outer_radius,
        } => yaml.push_str(&format!(
            "{indent}kind: ring\n{indent}inner_radius: {}\n{indent}outer_radius: {}\n",
            fmt_f32(inner_radius),
            fmt_f32(outer_radius)
        )),
    }
}

fn append_shape_yaml(yaml: &mut String, shape: ParticleShape2d, indent: &str) {
    match shape {
        ParticleShape2d::Circle { segments } => yaml.push_str(&format!(
            "{indent}kind: circle\n{indent}segments: {segments}\n"
        )),
        ParticleShape2d::Quad => yaml.push_str(&format!("{indent}kind: quad\n")),
        ParticleShape2d::Line { length } => yaml.push_str(&format!(
            "{indent}kind: line\n{indent}length: {}\n",
            fmt_f32(length)
        )),
    }
}

fn append_force_yaml(yaml: &mut String, force: ParticleForce2d, indent: &str) {
    match force {
        ParticleForce2d::Gravity { acceleration } => yaml.push_str(&format!(
            "{indent}- kind: gravity\n{indent}  acceleration: {{ x: {}, y: {} }}\n",
            fmt_f32(acceleration.x),
            fmt_f32(acceleration.y)
        )),
        ParticleForce2d::ConstantAcceleration { acceleration } => yaml.push_str(&format!(
            "{indent}- kind: constant_acceleration\n{indent}  acceleration: {{ x: {}, y: {} }}\n",
            fmt_f32(acceleration.x),
            fmt_f32(acceleration.y)
        )),
        ParticleForce2d::Drag { coefficient } => yaml.push_str(&format!(
            "{indent}- kind: drag\n{indent}  coefficient: {}\n",
            fmt_f32(coefficient)
        )),
        ParticleForce2d::Wind { velocity, strength } => yaml.push_str(&format!(
            "{indent}- kind: wind\n{indent}  velocity: {{ x: {}, y: {} }}\n{indent}  strength: {}\n",
            fmt_f32(velocity.x),
            fmt_f32(velocity.y),
            fmt_f32(strength)
        )),
    }
}

fn inline_curve_yaml(curve: &Curve1d) -> String {
    match curve {
        Curve1d::Constant(value) => format!("{{ kind: constant, value: {} }}", fmt_f32(*value)),
        Curve1d::Linear => "{ kind: linear }".to_owned(),
        Curve1d::EaseIn => "{ kind: ease_in }".to_owned(),
        Curve1d::EaseOut => "{ kind: ease_out }".to_owned(),
        Curve1d::EaseInOut => "{ kind: ease_in_out }".to_owned(),
        Curve1d::SmoothStep => "{ kind: smooth_step }".to_owned(),
        Curve1d::Custom { points } => {
            let points = points
                .iter()
                .map(|point| {
                    format!(
                        "{{ t: {}, value: {} }}",
                        fmt_f32(point.t),
                        fmt_f32(point.value)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{ kind: custom, points: [{points}] }}")
        }
    }
}

fn color_interpolation_name(interpolation: amigo_fx::ColorInterpolation) -> &'static str {
    match interpolation {
        amigo_fx::ColorInterpolation::LinearRgb => "linear_rgb",
        amigo_fx::ColorInterpolation::Step => "step",
    }
}

fn align_name(align: ParticleAlignMode2d) -> &'static str {
    match align {
        ParticleAlignMode2d::None => "none",
        ParticleAlignMode2d::Velocity => "velocity",
        ParticleAlignMode2d::Emitter => "emitter",
        ParticleAlignMode2d::Random => "random",
    }
}

fn color_to_hex(color: ColorRgba) -> String {
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        color_channel(color.r),
        color_channel(color.g),
        color_channel(color.b),
        color_channel(color.a)
    )
}

fn color_channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn fmt_f32(value: f32) -> String {
    if value.abs() < 0.000_001 {
        return "0.0".to_owned();
    }
    let mut formatted = format!("{value:.4}");
    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.push('0');
    }
    formatted
}

fn particle_shape_from_scene_command(shape: ParticleShape2dSceneCommand) -> ParticleShape2d {
    match shape {
        ParticleShape2dSceneCommand::Circle { segments } => ParticleShape2d::Circle { segments },
        ParticleShape2dSceneCommand::Quad => ParticleShape2d::Quad,
        ParticleShape2dSceneCommand::Line { length } => ParticleShape2d::Line { length },
    }
}

fn particle_align_from_scene_command(align: ParticleAlignMode2dSceneCommand) -> ParticleAlignMode2d {
    match align {
        ParticleAlignMode2dSceneCommand::None => ParticleAlignMode2d::None,
        ParticleAlignMode2dSceneCommand::Velocity => ParticleAlignMode2d::Velocity,
        ParticleAlignMode2dSceneCommand::Emitter => ParticleAlignMode2d::Emitter,
        ParticleAlignMode2dSceneCommand::Random => ParticleAlignMode2d::Random,
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
        registry.register(ParticlePreset2dService::default())?;
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
                align: ParticleAlignMode2d::Velocity,
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
    fn burst_at_spawns_particles_at_requested_position() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(false);
        command.emitter.initial_speed = 0.0;
        command.emitter.spawn_area = ParticleSpawnArea2d::Rect {
            size: Vec2::new(100.0, 100.0),
        };
        service.queue_emitter(command);

        assert!(service.burst_at("thruster", Vec2::new(42.0, -24.0), 3));
        service.tick(&[test_input()], 0.1);

        let draw = service.draw_commands();
        assert_eq!(draw.len(), 3);
        assert!(draw
            .iter()
            .all(|command| command.position == Vec2::new(42.0, -24.0)));
    }

    #[test]
    fn burst_at_rejects_invalid_position() {
        let service = Particle2dSceneService::default();
        service.queue_emitter(test_emitter(false));

        assert!(!service.burst_at("thruster", Vec2::new(f32::NAN, 0.0), 1));
        assert!(!service.burst_at("missing", Vec2::ZERO, 1));
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

    #[test]
    fn runtime_setters_update_jitter_direction_and_inheritance() {
        let service = Particle2dSceneService::default();
        service.queue_emitter(test_emitter(false));

        assert!(service.set_lifetime_jitter("thruster", 0.25));
        assert!(service.set_speed_jitter("thruster", 8.0));
        assert!(service.set_local_direction_radians("thruster", 1.5));
        assert!(service.set_inherit_parent_velocity("thruster", 0.35));

        let emitter = service.emitter("thruster").expect("emitter should exist");
        assert_eq!(emitter.emitter.lifetime_jitter, 0.25);
        assert_eq!(emitter.emitter.speed_jitter, 8.0);
        assert_eq!(emitter.emitter.local_direction_radians, 1.5);
        assert_eq!(emitter.emitter.inherit_parent_velocity, 0.35);
    }

    #[test]
    fn exports_particle_emitter_yaml_from_runtime_config() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(false);
        command.emitter.color_ramp = Some(ColorRamp::constant(ColorRgba::new(0.0, 1.0, 0.0, 1.0)));
        command.emitter.spawn_area = ParticleSpawnArea2d::Ring {
            inner_radius: 4.0,
            outer_radius: 12.0,
        };
        command.emitter.shape = ParticleShape2d::Line { length: 14.0 };
        command.emitter.align = ParticleAlignMode2d::Emitter;
        command.emitter.forces = vec![ParticleForce2d::Drag { coefficient: 0.5 }];
        service.queue_emitter(command);

        let yaml = service
            .emitter_yaml("thruster")
            .expect("emitter yaml should exist");

        assert!(yaml.contains("type: ParticleEmitter2D"));
        assert!(yaml.contains("color_ramp:"));
        assert!(yaml.contains("kind: ring"));
        assert!(yaml.contains("kind: line"));
        assert!(yaml.contains("align: emitter"));
        assert!(yaml.contains("kind: drag"));
    }

    #[test]
    fn align_none_keeps_particle_rotation_zero() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.align = ParticleAlignMode2d::None;
        command.emitter.local_direction_radians = 1.2;
        service.queue_emitter(command);

        service.tick(&[test_input()], 0.1);

        assert_eq!(service.draw_commands()[0].transform.rotation_radians, 0.0);
    }

    #[test]
    fn align_emitter_uses_emitter_rotation() {
        let service = Particle2dSceneService::default();
        let mut command = test_emitter(true);
        command.emitter.align = ParticleAlignMode2d::Emitter;
        command.emitter.local_direction_radians = 1.2;
        service.queue_emitter(command);

        service.tick(&[test_input()], 0.1);

        assert!((service.draw_commands()[0].transform.rotation_radians - 1.2).abs() < 0.001);
    }

    #[test]
    fn copy_emitter_config_replaces_target_emitter_and_clears_live_particles() {
        let service = Particle2dSceneService::default();
        let mut source = test_emitter(false);
        source.entity_id = SceneEntityId::new(2);
        source.entity_name = "source".to_owned();
        source.emitter.spawn_rate = 44.0;
        source.emitter.initial_speed = 33.0;
        source.emitter.shape = ParticleShape2d::Line { length: 18.0 };
        source.emitter.spawn_area = ParticleSpawnArea2d::Rect {
            size: Vec2::new(24.0, 8.0),
        };

        service.queue_emitter(test_emitter(true));
        service.queue_emitter(source);
        service.tick(&[test_input()], 0.2);
        assert!(service.particle_count("thruster") > 0);

        assert!(service.copy_emitter_config("source", "thruster"));

        let copied = service
            .emitter("thruster")
            .expect("target emitter should exist");
        assert_eq!(copied.emitter.spawn_rate, 44.0);
        assert_eq!(copied.emitter.initial_speed, 33.0);
        assert_eq!(copied.emitter.shape, ParticleShape2d::Line { length: 18.0 });
        assert_eq!(
            copied.emitter.spawn_area,
            ParticleSpawnArea2d::Rect {
                size: Vec2::new(24.0, 8.0)
            }
        );
        assert_eq!(service.particle_count("thruster"), 0);
    }
}
