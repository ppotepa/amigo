impl Particle2dSceneService {
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
            let input = input_lookup.get(entity_name).copied();
            if let Some(input) = input {
                state
                    .source_transforms
                    .insert(entity_name.to_owned(), input.source_transform);
            }
            let particles = state
                .particles
                .entry(command.entity_name.clone())
                .or_default();
            for particle in particles.iter_mut() {
                particle.previous_position = particle.position;
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
            let source_transform = if emitter.simulation_space == ParticleSimulationSpace2d::Source
            {
                state.source_transforms.get(entity_name).copied()
            } else {
                None
            };
            let latest_source_transform = state.source_transforms.get(entity_name).copied();
            let source_light_position = latest_source_transform
                .map(|transform| local_to_world_position(emitter.local_offset, transform));
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
                    previous_position: particle_position_for_draw(
                        particle.previous_position,
                        source_transform,
                    ),
                    position: particle_position_for_draw(particle.position, source_transform),
                    size,
                    color: ColorRgba::new(
                        sampled_color.r,
                        sampled_color.g,
                        sampled_color.b,
                        sampled_color.a * alpha,
                    ),
                    z_index: emitter.z_index,
                    shape: sample_particle_shape_over_lifetime(
                        &emitter.shape_over_lifetime,
                        particle.shape,
                        age_t,
                    ),
                    line_anchor: emitter.line_anchor,
                    blend_mode: emitter.blend_mode,
                    motion_stretch: emitter.motion_stretch,
                    material: emitter.material,
                    light: emitter.light,
                    light_position: emitter.light.and_then(|light| match light.mode {
                        ParticleLightMode2d::Source => source_light_position,
                        ParticleLightMode2d::Particle => None,
                    }),
                    transform: Transform2 {
                        rotation_radians: particle_rotation_for_align(
                            particle,
                            emitter.align,
                            source_transform,
                        ),
                        ..Transform2::default()
                    },
                });
            }
        }
        commands
    }
}
