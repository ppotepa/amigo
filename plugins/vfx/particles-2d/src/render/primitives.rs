use amigo_render_api::{
    LightReceiver2dBindingPrimitive, LightReceiverDarkPolicy2dPrimitive,
    LightReceiverGlobalLight2dPrimitive, LightSampleStrategy2dPrimitive, Particle2dPrimitive,
    ParticleBlendMode2dPrimitive, ParticleLight2dPrimitive, ParticleLightMode2dPrimitive,
    ParticleLineAnchor2dPrimitive, ParticleMaterial2dPrimitive,
    ParticleMaterialLightingMode2dPrimitive, ParticleMotionStretch2dPrimitive,
    ParticleShape2dPrimitive, RenderContribution2d, RenderPrimitive2d, Renderable2dCommon,
    Renderable2dItem, Renderable2dKind,
};
use amigo_camera_optics_plugin::api::CameraOpticalResponse2d;
use amigo_render_api::{
    LightContributionKind2d, LightEmitterKind2d, LightSource2dCommon, LightSource2dCommonParams,
};

use crate::{Particle2dDrawCommand, ParticleLight2d};

pub fn particle_draw_command_to_render_primitive(
    command: &Particle2dDrawCommand,
) -> RenderPrimitive2d {
    RenderPrimitive2d::ParticleBatch(Particle2dPrimitive {
        emitter_entity_name: command.emitter_entity_name.clone(),
        render_layer: command.render_layer.clone(),
        previous_position: command.previous_position,
        position: command.position,
        velocity: command.velocity,
        size: command.size,
        color: command.color,
        shape: match command.shape {
            crate::ParticleShape2d::Circle { segments } => {
                ParticleShape2dPrimitive::Circle { segments }
            }
            crate::ParticleShape2d::Quad => ParticleShape2dPrimitive::Quad,
            crate::ParticleShape2d::Line { length } => ParticleShape2dPrimitive::Line { length },
        },
        line_anchor: match command.line_anchor {
            crate::ParticleLineAnchor2d::Center => ParticleLineAnchor2dPrimitive::Center,
            crate::ParticleLineAnchor2d::Start => ParticleLineAnchor2dPrimitive::Start,
            crate::ParticleLineAnchor2d::End => ParticleLineAnchor2dPrimitive::End,
        },
        blend_mode: match command.blend_mode {
            crate::ParticleBlendMode2d::Alpha => ParticleBlendMode2dPrimitive::Alpha,
            crate::ParticleBlendMode2d::Additive => ParticleBlendMode2dPrimitive::Additive,
            crate::ParticleBlendMode2d::Multiply => ParticleBlendMode2dPrimitive::Multiply,
            crate::ParticleBlendMode2d::Screen => ParticleBlendMode2dPrimitive::Screen,
        },
        motion_stretch: command.motion_stretch.map(|stretch| ParticleMotionStretch2dPrimitive {
            enabled: stretch.enabled,
            velocity_scale: stretch.velocity_scale,
            max_length: stretch.max_length,
            shutter_seconds: stretch.shutter_seconds,
            tail_alpha: stretch.tail_alpha,
            head_alpha: stretch.head_alpha,
        }),
        material: ParticleMaterial2dPrimitive {
            lighting_mode: match command.material.lighting_mode {
                amigo_light_2d_plugin::Material2dLightingMode::Unlit => {
                    ParticleMaterialLightingMode2dPrimitive::Unlit
                }
                amigo_light_2d_plugin::Material2dLightingMode::DynamicLights => {
                    ParticleMaterialLightingMode2dPrimitive::DynamicLights
                }
                amigo_light_2d_plugin::Material2dLightingMode::LightMapSampled => {
                    ParticleMaterialLightingMode2dPrimitive::LightMapSampled
                }
                amigo_light_2d_plugin::Material2dLightingMode::LightGroupSampled => {
                    ParticleMaterialLightingMode2dPrimitive::LightGroupSampled
                }
            },
            receives_light: command.material.receives_light,
            light_response: command.material.light_response,
            light_receiver: command
                .material
                .light_receiver
                .as_ref()
                .map(|binding| LightReceiver2dBindingPrimitive {
                    groups: binding.groups.clone(),
                    source: binding.source.clone(),
                    channel: binding.channel.clone(),
                    sample_strategy: match binding.sample_strategy {
                        amigo_light_2d_plugin::LightSampleStrategy2d::Point => {
                            LightSampleStrategy2dPrimitive::Point
                        }
                        amigo_light_2d_plugin::LightSampleStrategy2d::Line => {
                            LightSampleStrategy2dPrimitive::Line
                        }
                    },
                    sample_points: binding.sample_points,
                    radius_px: binding.radius_px,
                    exposure: binding.exposure,
                    dark_policy: match binding.dark_policy {
                        amigo_light_2d_plugin::LightReceiverDarkPolicy2d::Transparent => {
                            LightReceiverDarkPolicy2dPrimitive::Transparent
                        }
                        amigo_light_2d_plugin::LightReceiverDarkPolicy2d::BaseColor => {
                            LightReceiverDarkPolicy2dPrimitive::BaseColor
                        }
                        amigo_light_2d_plugin::LightReceiverDarkPolicy2d::ShadowTint => {
                            LightReceiverDarkPolicy2dPrimitive::ShadowTint
                        }
                    },
                    global_lights: binding
                        .global_lights
                        .iter()
                        .map(|light| LightReceiverGlobalLight2dPrimitive {
                            id: light.id.clone(),
                            response: light.response,
                        })
                        .collect(),
                }),
        },
        light: command.light.map(|light| ParticleLight2dPrimitive {
            radius: light.radius,
            intensity: light.intensity,
            mode: match light.mode {
                crate::ParticleLightMode2d::Source => ParticleLightMode2dPrimitive::Source,
                crate::ParticleLightMode2d::Particle => ParticleLightMode2dPrimitive::Particle,
            },
            glow: light.glow,
        }),
        light_position: command.light_position,
        transform: command.transform,
    })
}

pub fn particle_draw_command_to_renderable_2d(
    command: &Particle2dDrawCommand,
) -> Renderable2dItem {
    Renderable2dItem::new(
        Renderable2dCommon::world(
            command.emitter_entity_name.clone(),
            "ParticleEmitter2D",
            command.render_layer.clone(),
            command.z_index,
            Renderable2dKind::Particle,
        ),
        particle_draw_command_to_render_primitive(command),
    )
}

pub fn particle_draw_command_to_light_contribution(
    command: &Particle2dDrawCommand,
) -> Option<RenderContribution2d> {
    let light = command.light?;
    Some(RenderContribution2d::light_source_2d(
        particle_draw_command_to_light_source(command, light),
    ))
}

fn particle_draw_command_to_light_source(
    command: &Particle2dDrawCommand,
    light: ParticleLight2d,
) -> LightSource2dCommon {
    let active = light.intensity > 0.001 && command.color.a > 0.001 && light.radius > 0.001;
    let position = command.light_position.unwrap_or(command.position);
    let params = LightSource2dCommonParams {
        owner: command.emitter_entity_name.clone(),
        component_kind: "ParticleEmitter2D".to_owned(),
        emitter_kind: LightEmitterKind2d::ParticleLight,
        emitter_id: Some(command.emitter_entity_name.clone()),
        render_layer: Some(command.render_layer.clone()),
        color_rgba: Some([command.color.r, command.color.g, command.color.b, command.color.a]),
        intensity: Some(light.intensity),
        effective_intensity: Some(light.intensity * command.color.a),
        response: Some(1.0),
        camera_response: Some(particle_light_camera_response(light)),
        bloom: None,
        radius_px: Some(light.radius),
        falloff: None,
        distance_m: None,
        z_depth: None,
        contributions: particle_light_contributions(light),
        reason: if active {
            "particle_light_active".to_owned()
        } else {
            "particle_light_zero_intensity".to_owned()
        },
        position_px: Some([position.x, position.y]),
    };
    if active {
        LightSource2dCommon::active(params)
    } else {
        LightSource2dCommon::skipped(params)
    }
}

fn particle_light_camera_response(light: ParticleLight2d) -> CameraOpticalResponse2d {
    CameraOpticalResponse2d {
        enabled: light.intensity > 0.0 && light.glow,
        intensity: light.intensity,
        bloom: if light.glow { light.intensity * 0.35 } else { 0.0 },
        glare: light.intensity * 0.2,
        ghosting: 0.0,
        streaks: 0.0,
        chromatic_smear: 0.0,
        dirt_response: 0.0,
        halation: if light.glow { light.intensity * 0.15 } else { 0.0 },
        threshold: 0.0,
    }
    .normalized()
}

fn particle_light_contributions(light: ParticleLight2d) -> Vec<LightContributionKind2d> {
    let mut contributions = vec![LightContributionKind2d::LightingEmit];
    if light.glow && light.intensity > 0.0 {
        contributions.push(LightContributionKind2d::BloomSource);
        contributions.push(LightContributionKind2d::CameraFxSource);
    }
    contributions
}
