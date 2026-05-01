fn tick_particle_profile_controller(
    behaviors: &BehaviorSceneService,
    actions: &InputActionService,
    input: &InputState,
    particles: &Particle2dSceneService,
    behavior_key: &str,
    config: &amigo_behavior::ParticleProfileControllerBehavior,
    delta_seconds: f32,
) {
    let intensity = actions.axis(input, &config.action).abs().clamp(0.0, 1.0);
    let active = intensity > 0.01;
    let max_hold_seconds = config.max_hold_seconds.max(0.0);
    let (previous_hold, hold_seconds) =
        behaviors.tick_hold_seconds(behavior_key, active, delta_seconds, max_hold_seconds);

    particles.set_active(&config.emitter, active);
    particles.set_intensity(&config.emitter, if active { intensity } else { 0.0 });

    if !active {
        return;
    }

    let Some(phase) = active_particle_profile_phase(&config.phases, hold_seconds) else {
        return;
    };
    apply_particle_profile_phase(
        particles,
        &config.emitter,
        phase,
        previous_hold,
        hold_seconds,
        intensity,
    );
}

fn active_particle_profile_phase(
    phases: &[ParticleProfilePhase],
    hold_seconds: f32,
) -> Option<&ParticleProfilePhase> {
    phases
        .iter()
        .find(|phase| hold_seconds >= phase.start_seconds && hold_seconds < phase.end_seconds)
        .or_else(|| {
            phases
                .iter()
                .filter(|phase| hold_seconds >= phase.start_seconds)
                .max_by(|left, right| left.start_seconds.total_cmp(&right.start_seconds))
        })
        .or_else(|| phases.first())
}

fn apply_particle_profile_phase(
    particles: &Particle2dSceneService,
    emitter: &str,
    phase: &ParticleProfilePhase,
    previous_hold: f32,
    hold_seconds: f32,
    intensity: f32,
) {
    let phase_duration = (phase.end_seconds - phase.start_seconds).max(f32::EPSILON);
    let phase_t = ((hold_seconds - phase.start_seconds) / phase_duration).clamp(0.0, 1.0);
    let previous_phase_t = ((previous_hold - phase.start_seconds) / phase_duration).clamp(0.0, 1.0);
    let noise = particle_profile_noise(hold_seconds);

    if let Some(mode) = phase.velocity_mode {
        particles.set_velocity_mode(emitter, particle_velocity_mode_from_profile(mode));
    }
    if phase.clear_forces {
        particles.clear_forces(emitter);
    }
    if let Some(ramp) = phase.color_ramp.as_ref() {
        particles.set_color_ramp(emitter, ramp.clone());
    }

    apply_profile_scalar(&phase.spawn_rate, phase_t, intensity, noise, |value| {
        particles.set_spawn_rate(emitter, value)
    });
    apply_profile_scalar(&phase.lifetime, phase_t, intensity, noise, |value| {
        particles.set_particle_lifetime(emitter, value)
    });
    apply_profile_scalar(&phase.lifetime_jitter, phase_t, intensity, noise, |value| {
        particles.set_lifetime_jitter(emitter, value)
    });
    apply_profile_scalar(&phase.speed, phase_t, intensity, noise, |value| {
        particles.set_initial_speed(emitter, value)
    });
    apply_profile_scalar(&phase.speed_jitter, phase_t, intensity, noise, |value| {
        particles.set_speed_jitter(emitter, value)
    });
    apply_profile_scalar(&phase.spread_degrees, phase_t, intensity, noise, |value| {
        particles.set_spread_radians(emitter, value.to_radians())
    });
    apply_profile_scalar(&phase.initial_size, phase_t, intensity, noise, |value| {
        particles.set_initial_size(emitter, value)
    });
    apply_profile_scalar(&phase.final_size, phase_t, intensity, noise, |value| {
        particles.set_final_size(emitter, value)
    });
    apply_profile_scalar(&phase.spawn_area_line, phase_t, intensity, noise, |value| {
        particles.set_spawn_area(
            emitter,
            ParticleSpawnArea2d::Line {
                length: value.max(0.0),
            },
        )
    });
    apply_profile_scalar(&phase.shape_line, phase_t, intensity, noise, |value| {
        particles.set_shape(
            emitter,
            ParticleShape2d::Line {
                length: value.max(0.0),
            },
        )
    });

    if phase.shape_circle_weight.is_some()
        || phase.shape_line_weight.is_some()
        || phase.shape_quad_weight.is_some()
    {
        let line_length = phase
            .shape_line
            .as_ref()
            .map(|scalar| sample_profile_scalar(scalar, phase_t, intensity, noise).max(0.0))
            .unwrap_or(4.0);
        let mut choices = Vec::new();
        let circle_weight = phase
            .shape_circle_weight
            .as_ref()
            .map(|scalar| sample_profile_scalar(scalar, phase_t, intensity, noise))
            .unwrap_or(0.0);
        let line_weight = phase
            .shape_line_weight
            .as_ref()
            .map(|scalar| sample_profile_scalar(scalar, phase_t, intensity, noise))
            .unwrap_or(0.0);
        let quad_weight = phase
            .shape_quad_weight
            .as_ref()
            .map(|scalar| sample_profile_scalar(scalar, phase_t, intensity, noise))
            .unwrap_or(0.0);

        if circle_weight > 0.0 {
            choices.push(WeightedParticleShape2d {
                shape: ParticleShape2d::Circle { segments: 6 },
                weight: circle_weight,
            });
        }
        if line_weight > 0.0 {
            choices.push(WeightedParticleShape2d {
                shape: ParticleShape2d::Line {
                    length: line_length,
                },
                weight: line_weight,
            });
        }
        if quad_weight > 0.0 {
            choices.push(WeightedParticleShape2d {
                shape: ParticleShape2d::Quad,
                weight: quad_weight,
            });
        }
        particles.set_shape_choices(emitter, choices);
    }

    apply_profile_curve4(
        particles,
        emitter,
        "size",
        phase.size_curve.as_ref(),
        phase_t,
        intensity,
        noise,
    );
    apply_profile_curve4(
        particles,
        emitter,
        "speed",
        phase.speed_curve.as_ref(),
        phase_t,
        intensity,
        noise,
    );
    apply_profile_curve4(
        particles,
        emitter,
        "alpha",
        phase.alpha_curve.as_ref(),
        phase_t,
        intensity,
        noise,
    );

    if let Some(burst) = phase.burst.as_ref() {
        let rate_hz = burst.rate_hz.max(0.0);
        if rate_hz > 0.0
            && phase_t >= burst.threshold.clamp(0.0, 1.0)
            && (previous_phase_t * rate_hz).floor() < (phase_t * rate_hz).floor()
        {
            let min_count = burst.min_count.min(burst.max_count);
            let max_count = burst.max_count.max(min_count);
            let count = min_count
                + ((max_count - min_count) as f32 * (phase_t * 0.72 + noise * 0.28)).round()
                    as usize;
            particles.burst(emitter, count);
        }
    }
}

fn apply_profile_scalar(
    scalar: &Option<ParticleProfileScalar>,
    phase_t: f32,
    intensity: f32,
    noise: f32,
    apply: impl FnOnce(f32) -> bool,
) {
    if let Some(scalar) = scalar.as_ref() {
        let _ = apply(sample_profile_scalar(scalar, phase_t, intensity, noise));
    }
}

fn sample_profile_scalar(
    scalar: &ParticleProfileScalar,
    phase_t: f32,
    intensity: f32,
    noise: f32,
) -> f32 {
    let t = scalar.curve.sample(phase_t.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    scalar.from
        + (scalar.to - scalar.from) * t
        + scalar.intensity_scale * intensity
        + scalar.noise_scale * noise
}

fn apply_profile_curve4(
    particles: &Particle2dSceneService,
    emitter: &str,
    name: &str,
    curve: Option<&ParticleProfileCurve4>,
    phase_t: f32,
    intensity: f32,
    noise: f32,
) {
    if let Some(curve) = curve {
        let _ = particles.set_curve4(
            emitter,
            name,
            sample_profile_scalar(&curve.v0, phase_t, intensity, noise),
            sample_profile_scalar(&curve.v1, phase_t, intensity, noise),
            sample_profile_scalar(&curve.v2, phase_t, intensity, noise),
            sample_profile_scalar(&curve.v3, phase_t, intensity, noise),
        );
    }
}

fn particle_velocity_mode_from_profile(
    mode: ParticleProfileVelocityMode,
) -> ParticleVelocityMode2d {
    match mode {
        ParticleProfileVelocityMode::Free => ParticleVelocityMode2d::Free,
        ParticleProfileVelocityMode::SourceInertial => ParticleVelocityMode2d::SourceInertial,
    }
}

fn particle_profile_noise(hold_seconds: f32) -> f32 {
    let a = triangle_wave(hold_seconds * 3.7 + 0.19);
    let b = triangle_wave(hold_seconds * 7.1 + 0.43);
    let c = triangle_wave(hold_seconds * 11.3 + 0.71);
    (a * 0.52 + b * 0.32 + c * 0.16).clamp(0.0, 1.0)
}

fn triangle_wave(value: f32) -> f32 {
    let t = value - value.floor();
    if t < 0.5 { t * 2.0 } else { (1.0 - t) * 2.0 }
}

