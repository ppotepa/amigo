use super::*;

pub(super) fn collect_particle_light_sources(
    particles: &[amigo_particles_2d_plugin::Particle2dDrawCommand],
    sources: &mut Vec<LightSource2dCommon>,
) {
    for particle in particles.iter().take(MAX_PARTICLE_LIGHT_SOURCES) {
        if let Some(light) = particle.light {
            let active =
                light.intensity > 0.001 && particle.color.a > 0.001 && light.radius > 0.001;
            let position = particle.light_position.unwrap_or(particle.position);
            let position_px = Some([position.x, position.y]);
            let common = if active {
                active_light_source!(
                    particle.emitter_entity_name.clone(),
                    "ParticleEmitter2D",
                    LightEmitterKind2d::ParticleLight,
                    Some(particle.emitter_entity_name.clone()),
                    Some(particle.render_layer.clone()),
                    Some(color_rgba(particle.color)),
                    Some(light.intensity),
                    Some(light.intensity * particle.color.a),
                    Some(1.0),
                    Some(particle_light_camera_response(light)),
                    None,
                    None,
                    Some(light.radius),
                    None,
                    None,
                    particle_light_contributions(light),
                    "particle_light_active",
                    position_px,
                )
            } else {
                skipped_light_source!(
                    particle.emitter_entity_name.clone(),
                    "ParticleEmitter2D",
                    LightEmitterKind2d::ParticleLight,
                    Some(particle.emitter_entity_name.clone()),
                    Some(particle.render_layer.clone()),
                    Some(color_rgba(particle.color)),
                    Some(light.intensity),
                    Some(light.intensity * particle.color.a),
                    Some(1.0),
                    Some(particle_light_camera_response(light)),
                    None,
                    None,
                    Some(light.radius),
                    None,
                    None,
                    particle_light_contributions(light),
                    "particle_light_zero_intensity",
                    position_px,
                )
            };
            sources.push(common);
        }
    }
}

fn particle_light_camera_response(
    light: amigo_particles_2d_plugin::ParticleLight2d,
) -> CameraOpticalResponse2d {
    CameraOpticalResponse2d {
        enabled: light.intensity > 0.0 && light.glow,
        intensity: light.intensity,
        bloom: if light.glow {
            light.intensity * 0.35
        } else {
            0.0
        },
        glare: light.intensity * 0.2,
        ghosting: 0.0,
        streaks: 0.0,
        chromatic_smear: 0.0,
        dirt_response: 0.0,
        halation: if light.glow {
            light.intensity * 0.15
        } else {
            0.0
        },
        threshold: 0.0,
    }
    .normalized()
}

fn particle_light_contributions(
    light: amigo_particles_2d_plugin::ParticleLight2d,
) -> Vec<LightContributionKind2d> {
    let mut contributions = vec![LightContributionKind2d::LightingEmit];
    if light.glow && light.intensity > 0.0 {
        contributions.push(LightContributionKind2d::BloomSource);
        contributions.push(LightContributionKind2d::CameraFxSource);
    }
    contributions
}
