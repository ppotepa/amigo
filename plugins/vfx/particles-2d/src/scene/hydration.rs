use amigo_fx::{ColorInterpolation, ColorRamp, ColorStop};
use amigo_math::{ColorRgba, Curve1d, CurvePoint1d, Vec2};
use amigo_scene::{
    ColorInterpolationSceneDocument, ColorRampSceneDocument, Curve1dSceneDocument,
    LightReceiver2dBindingSceneCommand, LightReceiver2dBindingSceneDocument,
    LightReceiverDarkPolicy2dSceneCommand, LightReceiverDarkPolicy2dSceneDocument,
    LightReceiverGlobalLight2dSceneCommand, LightReceiverGlobalLight2dSceneDocument,
    LightSampleStrategy2dSceneCommand, LightSampleStrategy2dSceneDocument,
    Material2dLightingModeSceneCommand, Material2dLightingModeSceneDocument,
    ParticleAlignMode2dSceneCommand, ParticleAlignMode2dSceneDocument,
    ParticleBlendMode2dSceneCommand, ParticleBlendMode2dSceneDocument,
    ParticleEmitter2dSceneCommand, ParticleForce2dSceneCommand, ParticleForce2dSceneDocument,
    ParticleLight2dSceneCommand, ParticleLightMode2dSceneCommand, ParticleLightMode2dSceneDocument,
    ParticleLineAnchor2dSceneCommand, ParticleLineAnchor2dSceneDocument,
    ParticleMaterial2dSceneCommand, ParticleMotionStretch2dSceneCommand,
    ParticleShape2dSceneCommand, ParticleShape2dSceneDocument, ParticleShapeChoice2dSceneCommand,
    ParticleShapeKeyframe2dSceneCommand, ParticleSimulationSpace2dSceneCommand,
    ParticleSimulationSpace2dSceneDocument, ParticleSpawnArea2dSceneCommand,
    ParticleSpawnArea2dSceneDocument, ParticleVelocityMode2dSceneCommand,
    ParticleVelocityMode2dSceneDocument, PluginComponentHydrationContext, PluginComponentHydrator,
    SceneCommand, SceneDocumentError, SceneDocumentResult, SceneVec2Document,
};

use super::ParticleEmitter2dDocument;

#[derive(Default)]
pub struct ParticleEmitter2dPluginComponentHydrator;

impl PluginComponentHydrator for ParticleEmitter2dPluginComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.vfx.particles-2d"
    }

    fn component_type(&self) -> &'static str {
        "amigo.vfx.particles-2d.ParticleEmitter2D"
    }

    fn hydrate_plugin_payload(
        &self,
        ctx: PluginComponentHydrationContext<'_>,
    ) -> SceneDocumentResult<()> {
        let Some(document) = ctx
            .payload
            .as_any()
            .downcast_ref::<ParticleEmitter2dDocument>()
        else {
            return Err(SceneDocumentError::Hydration {
                scene_id: ctx.document.scene.id.clone(),
                entity_id: ctx.entity.id.clone(),
                component_kind: ctx.component_type.to_owned(),
                message: "ParticleEmitter2D plugin hydrator received wrong payload".to_owned(),
            });
        };

        push_particle_emitter_command(
            document,
            ctx.source_mod,
            &ctx.document.scene.id,
            &ctx.entity.id,
            ctx.entity_name,
            ctx.commands,
        )
    }
}

fn push_particle_emitter_command(
    document: &ParticleEmitter2dDocument,
    source_mod: &str,
    scene_id: &str,
    entity_id: &str,
    entity_name: &str,
    commands: &mut Vec<SceneCommand>,
) -> SceneDocumentResult<()> {
    commands.push(SceneCommand::Plugin {
        command: amigo_scene::particle_emitter_2d_plugin_scene_command(
            ParticleEmitter2dSceneCommand {
                source_mod: source_mod.to_owned(),
                entity_name: entity_name.to_owned(),
                render_layer: document.render_layer.clone(),
                attached_to: document.attached_to.clone(),
                local_offset: vec2_from_document(document.local_offset),
                local_direction_radians: document.local_direction_degrees.to_radians(),
                spawn_area: particle_spawn_area_from_document(document.spawn_area.as_ref()),
                active: document.active,
                spawn_rate: document.spawn_rate,
                max_particles: document.max_particles,
                particle_lifetime: document.particle_lifetime,
                lifetime_jitter: document.lifetime_jitter,
                initial_speed: document.initial_speed,
                speed_jitter: document.speed_jitter,
                spread_radians: document.spread_degrees.to_radians(),
                inherit_parent_velocity: document.inherit_parent_velocity,
                velocity_mode: particle_velocity_mode_from_document(document.velocity_mode),
                simulation_space: particle_simulation_space_from_document(
                    document.simulation_space,
                ),
                initial_size: document.initial_size,
                final_size: document.final_size,
                size_jitter: document.size_jitter.max(0.0),
                color: parse_optional_color_rgba_hex(
                    document.color.as_deref(),
                    scene_id,
                    entity_id,
                    "ParticleEmitter2D",
                )?
                .unwrap_or(ColorRgba::WHITE),
                color_ramp: document
                    .color_ramp
                    .as_ref()
                    .map(|color_ramp| {
                        color_ramp_from_document(
                            color_ramp,
                            scene_id,
                            entity_id,
                            "ParticleEmitter2D",
                        )
                    })
                    .transpose()?,
                z_index: document.z_index,
                shape: particle_shape_from_document(document.shape.as_ref()),
                shape_choices: document
                    .shape_choices
                    .iter()
                    .map(|choice| ParticleShapeChoice2dSceneCommand {
                        shape: particle_shape_from_document(Some(&choice.shape)),
                        weight: choice.weight.max(0.0),
                    })
                    .collect(),
                shape_over_lifetime: document
                    .shape_over_lifetime
                    .iter()
                    .map(|keyframe| ParticleShapeKeyframe2dSceneCommand {
                        t: keyframe.t.clamp(0.0, 1.0),
                        shape: particle_shape_from_document(Some(&keyframe.shape)),
                    })
                    .collect(),
                line_anchor: particle_line_anchor_from_document(document.line_anchor),
                align: particle_align_from_document(document.align),
                blend_mode: particle_blend_from_document(document.blend_mode),
                motion_stretch: document.motion_stretch.as_ref().map(|motion_stretch| {
                    ParticleMotionStretch2dSceneCommand {
                        enabled: motion_stretch.enabled,
                        velocity_scale: motion_stretch.velocity_scale.max(0.0),
                        max_length: motion_stretch.max_length.max(0.0),
                        shutter_seconds: motion_stretch.shutter_seconds.max(0.0),
                        tail_alpha: motion_stretch.tail_alpha.clamp(0.0, 1.0),
                        head_alpha: motion_stretch.head_alpha.clamp(0.0, 1.0),
                    }
                }),
                material: document
                    .material
                    .as_ref()
                    .map(|material| ParticleMaterial2dSceneCommand {
                        lighting_mode: particle_lighting_mode_from_document(
                            material.lighting_mode,
                            material.receives_light,
                            material.light_receiver.as_ref(),
                        ),
                        receives_light: material.receives_light,
                        light_response: material.light_response.max(0.0),
                        light_receiver: material
                            .light_receiver
                            .as_ref()
                            .map(light_receiver_binding_from_document),
                    })
                    .unwrap_or(ParticleMaterial2dSceneCommand {
                        lighting_mode: Material2dLightingModeSceneCommand::Unlit,
                        receives_light: false,
                        light_response: 1.0,
                        light_receiver: None,
                    }),
                light: document.light.map(|light| ParticleLight2dSceneCommand {
                    radius: light.radius.max(0.0),
                    intensity: light.intensity.max(0.0),
                    mode: particle_light_mode_from_document(light.mode),
                    glow: light.glow,
                }),
                emission_rate_curve: curve1d_from_optional_document(
                    document.emission_rate_curve.as_ref(),
                ),
                size_curve: curve1d_from_optional_document(document.size_curve.as_ref()),
                alpha_curve: document
                    .alpha_curve
                    .as_ref()
                    .map(curve1d_from_document)
                    .unwrap_or(Curve1d::Constant(1.0)),
                speed_curve: document
                    .speed_curve
                    .as_ref()
                    .map(curve1d_from_document)
                    .unwrap_or(Curve1d::Constant(1.0)),
                forces: document
                    .forces
                    .iter()
                    .map(particle_force_from_document)
                    .collect(),
            },
        ),
    });

    Ok(())
}

fn vec2_from_document(value: SceneVec2Document) -> Vec2 {
    Vec2::new(value.x, value.y)
}

fn color_ramp_from_document(
    document: &ColorRampSceneDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<ColorRamp> {
    Ok(ColorRamp {
        interpolation: match document.interpolation {
            ColorInterpolationSceneDocument::LinearRgb => ColorInterpolation::LinearRgb,
            ColorInterpolationSceneDocument::Step => ColorInterpolation::Step,
        },
        stops: document
            .stops
            .iter()
            .map(|stop| {
                Ok(ColorStop {
                    t: stop.t,
                    color: parse_color_rgba_hex(&stop.color, scene_id, entity_id, component_kind)?,
                })
            })
            .collect::<SceneDocumentResult<Vec<_>>>()?,
    })
}

fn particle_shape_from_document(
    document: Option<&ParticleShape2dSceneDocument>,
) -> ParticleShape2dSceneCommand {
    match document {
        Some(ParticleShape2dSceneDocument::Circle { segments }) => {
            ParticleShape2dSceneCommand::Circle {
                segments: (*segments).max(3),
            }
        }
        Some(ParticleShape2dSceneDocument::Quad) => ParticleShape2dSceneCommand::Quad,
        Some(ParticleShape2dSceneDocument::Line { length }) => {
            ParticleShape2dSceneCommand::Line { length: *length }
        }
        None => ParticleShape2dSceneCommand::Circle { segments: 8 },
    }
}

fn particle_line_anchor_from_document(
    document: Option<ParticleLineAnchor2dSceneDocument>,
) -> ParticleLineAnchor2dSceneCommand {
    match document {
        Some(ParticleLineAnchor2dSceneDocument::Start) => ParticleLineAnchor2dSceneCommand::Start,
        Some(ParticleLineAnchor2dSceneDocument::End) => ParticleLineAnchor2dSceneCommand::End,
        Some(ParticleLineAnchor2dSceneDocument::Center) | None => {
            ParticleLineAnchor2dSceneCommand::Center
        }
    }
}

fn particle_align_from_document(
    document: Option<ParticleAlignMode2dSceneDocument>,
) -> ParticleAlignMode2dSceneCommand {
    match document {
        Some(ParticleAlignMode2dSceneDocument::None) => ParticleAlignMode2dSceneCommand::None,
        Some(ParticleAlignMode2dSceneDocument::Emitter) => ParticleAlignMode2dSceneCommand::Emitter,
        Some(ParticleAlignMode2dSceneDocument::Random) => ParticleAlignMode2dSceneCommand::Random,
        Some(ParticleAlignMode2dSceneDocument::Velocity) | None => {
            ParticleAlignMode2dSceneCommand::Velocity
        }
    }
}

fn particle_blend_from_document(
    document: Option<ParticleBlendMode2dSceneDocument>,
) -> ParticleBlendMode2dSceneCommand {
    match document {
        Some(ParticleBlendMode2dSceneDocument::Additive) => {
            ParticleBlendMode2dSceneCommand::Additive
        }
        Some(ParticleBlendMode2dSceneDocument::Multiply) => {
            ParticleBlendMode2dSceneCommand::Multiply
        }
        Some(ParticleBlendMode2dSceneDocument::Screen) => ParticleBlendMode2dSceneCommand::Screen,
        Some(ParticleBlendMode2dSceneDocument::Alpha) | None => {
            ParticleBlendMode2dSceneCommand::Alpha
        }
    }
}

fn particle_velocity_mode_from_document(
    document: Option<ParticleVelocityMode2dSceneDocument>,
) -> ParticleVelocityMode2dSceneCommand {
    match document {
        Some(ParticleVelocityMode2dSceneDocument::SourceInertial) => {
            ParticleVelocityMode2dSceneCommand::SourceInertial
        }
        Some(ParticleVelocityMode2dSceneDocument::Free) | None => {
            ParticleVelocityMode2dSceneCommand::Free
        }
    }
}

fn particle_simulation_space_from_document(
    document: Option<ParticleSimulationSpace2dSceneDocument>,
) -> ParticleSimulationSpace2dSceneCommand {
    match document {
        Some(ParticleSimulationSpace2dSceneDocument::Source) => {
            ParticleSimulationSpace2dSceneCommand::Source
        }
        Some(ParticleSimulationSpace2dSceneDocument::World) | None => {
            ParticleSimulationSpace2dSceneCommand::World
        }
    }
}

fn particle_light_mode_from_document(
    document: ParticleLightMode2dSceneDocument,
) -> ParticleLightMode2dSceneCommand {
    match document {
        ParticleLightMode2dSceneDocument::Source => ParticleLightMode2dSceneCommand::Source,
        ParticleLightMode2dSceneDocument::Particle => ParticleLightMode2dSceneCommand::Particle,
    }
}

fn particle_spawn_area_from_document(
    document: Option<&ParticleSpawnArea2dSceneDocument>,
) -> ParticleSpawnArea2dSceneCommand {
    match document {
        Some(ParticleSpawnArea2dSceneDocument::Point) | None => {
            ParticleSpawnArea2dSceneCommand::Point
        }
        Some(ParticleSpawnArea2dSceneDocument::Line { length }) => {
            ParticleSpawnArea2dSceneCommand::Line { length: *length }
        }
        Some(ParticleSpawnArea2dSceneDocument::Rect { size }) => {
            ParticleSpawnArea2dSceneCommand::Rect {
                size: vec2_from_document(*size),
            }
        }
        Some(ParticleSpawnArea2dSceneDocument::Circle { radius }) => {
            ParticleSpawnArea2dSceneCommand::Circle { radius: *radius }
        }
        Some(ParticleSpawnArea2dSceneDocument::Ring {
            inner_radius,
            outer_radius,
        }) => ParticleSpawnArea2dSceneCommand::Ring {
            inner_radius: *inner_radius,
            outer_radius: *outer_radius,
        },
    }
}

fn particle_force_from_document(
    document: &ParticleForce2dSceneDocument,
) -> ParticleForce2dSceneCommand {
    match document {
        ParticleForce2dSceneDocument::Gravity { acceleration } => {
            ParticleForce2dSceneCommand::Gravity {
                acceleration: vec2_from_document(*acceleration),
            }
        }
        ParticleForce2dSceneDocument::ConstantAcceleration { acceleration } => {
            ParticleForce2dSceneCommand::ConstantAcceleration {
                acceleration: vec2_from_document(*acceleration),
            }
        }
        ParticleForce2dSceneDocument::Drag { coefficient } => ParticleForce2dSceneCommand::Drag {
            coefficient: *coefficient,
        },
        ParticleForce2dSceneDocument::Wind { velocity, strength } => {
            ParticleForce2dSceneCommand::Wind {
                velocity: vec2_from_document(*velocity),
                strength: *strength,
            }
        }
    }
}

fn light_receiver_binding_from_document(
    binding: &LightReceiver2dBindingSceneDocument,
) -> LightReceiver2dBindingSceneCommand {
    LightReceiver2dBindingSceneCommand {
        groups: binding.groups.clone(),
        source: binding.source.clone(),
        channel: binding.channel.clone(),
        sample_strategy: light_sample_strategy_from_document(binding.sample_strategy),
        sample_points: binding.sample_points.clamp(1, 9),
        radius_px: binding.radius_px.max(0.0),
        exposure: binding.exposure.max(0.0),
        dark_policy: light_receiver_dark_policy_from_document(binding.dark_policy),
        global_lights: binding
            .global_lights
            .iter()
            .map(light_receiver_global_light_from_document)
            .collect(),
    }
}

fn particle_lighting_mode_from_document(
    explicit: Option<Material2dLightingModeSceneDocument>,
    receives_light: bool,
    receiver: Option<&LightReceiver2dBindingSceneDocument>,
) -> Material2dLightingModeSceneCommand {
    if let Some(mode) = explicit {
        return match mode {
            Material2dLightingModeSceneDocument::Unlit => Material2dLightingModeSceneCommand::Unlit,
            Material2dLightingModeSceneDocument::DynamicLights => {
                Material2dLightingModeSceneCommand::DynamicLights
            }
            Material2dLightingModeSceneDocument::LightmapSampled => {
                Material2dLightingModeSceneCommand::LightMapSampled
            }
            Material2dLightingModeSceneDocument::LightGroupSampled => {
                Material2dLightingModeSceneCommand::LightGroupSampled
            }
        };
    }

    let Some(receiver) = receiver else {
        return if receives_light {
            Material2dLightingModeSceneCommand::DynamicLights
        } else {
            Material2dLightingModeSceneCommand::Unlit
        };
    };

    if !receiver.groups.is_empty() {
        Material2dLightingModeSceneCommand::LightGroupSampled
    } else if !receiver.source.is_empty() && !receiver.channel.is_empty() {
        Material2dLightingModeSceneCommand::LightMapSampled
    } else if receives_light {
        Material2dLightingModeSceneCommand::DynamicLights
    } else {
        Material2dLightingModeSceneCommand::Unlit
    }
}

fn light_sample_strategy_from_document(
    strategy: LightSampleStrategy2dSceneDocument,
) -> LightSampleStrategy2dSceneCommand {
    match strategy {
        LightSampleStrategy2dSceneDocument::Point => LightSampleStrategy2dSceneCommand::Point,
        LightSampleStrategy2dSceneDocument::Line => LightSampleStrategy2dSceneCommand::Line,
    }
}

fn light_receiver_dark_policy_from_document(
    policy: LightReceiverDarkPolicy2dSceneDocument,
) -> LightReceiverDarkPolicy2dSceneCommand {
    match policy {
        LightReceiverDarkPolicy2dSceneDocument::Transparent => {
            LightReceiverDarkPolicy2dSceneCommand::Transparent
        }
        LightReceiverDarkPolicy2dSceneDocument::BaseColor => {
            LightReceiverDarkPolicy2dSceneCommand::BaseColor
        }
        LightReceiverDarkPolicy2dSceneDocument::ShadowTint => {
            LightReceiverDarkPolicy2dSceneCommand::ShadowTint
        }
    }
}

fn light_receiver_global_light_from_document(
    global_light: &LightReceiverGlobalLight2dSceneDocument,
) -> LightReceiverGlobalLight2dSceneCommand {
    LightReceiverGlobalLight2dSceneCommand {
        id: global_light.id.clone(),
        response: global_light.response.max(0.0),
    }
}

fn curve1d_from_optional_document(document: Option<&Curve1dSceneDocument>) -> Curve1d {
    document
        .map(curve1d_from_document)
        .unwrap_or(Curve1d::Linear)
}

fn curve1d_from_document(document: &Curve1dSceneDocument) -> Curve1d {
    match document {
        Curve1dSceneDocument::Constant { value } => Curve1d::Constant(*value),
        Curve1dSceneDocument::Linear => Curve1d::Linear,
        Curve1dSceneDocument::EaseIn => Curve1d::EaseIn,
        Curve1dSceneDocument::EaseOut => Curve1d::EaseOut,
        Curve1dSceneDocument::EaseInOut => Curve1d::EaseInOut,
        Curve1dSceneDocument::SmoothStep => Curve1d::SmoothStep,
        Curve1dSceneDocument::Custom { points } => Curve1d::Custom {
            points: points
                .iter()
                .map(|point| CurvePoint1d {
                    t: point.t,
                    value: point.value,
                })
                .collect(),
        },
    }
}

fn parse_optional_color_rgba_hex(
    value: Option<&str>,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<Option<ColorRgba>> {
    value
        .map(|value| parse_color_rgba_hex(value, scene_id, entity_id, component_kind))
        .transpose()
}

fn parse_color_rgba_hex(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<ColorRgba> {
    let value = value.trim();
    let hex = value.strip_prefix('#').unwrap_or(value);

    let (r, g, b, a) = match hex.len() {
        6 => (
            parse_hex_channel(&hex[0..2], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[2..4], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[4..6], value, scene_id, entity_id, component_kind)?,
            255,
        ),
        8 => (
            parse_hex_channel(&hex[0..2], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[2..4], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[4..6], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[6..8], value, scene_id, entity_id, component_kind)?,
        ),
        _ => {
            return Err(SceneDocumentError::Hydration {
                scene_id: scene_id.to_owned(),
                entity_id: entity_id.to_owned(),
                component_kind: component_kind.to_owned(),
                message: format!(
                    "expected albedo color `{value}` to use #RRGGBB or #RRGGBBAA syntax"
                ),
            });
        }
    };

    Ok(ColorRgba::new(
        channel_to_f32(r),
        channel_to_f32(g),
        channel_to_f32(b),
        channel_to_f32(a),
    ))
}

fn parse_hex_channel(
    channel: &str,
    raw_value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<u8> {
    u8::from_str_radix(channel, 16).map_err(|_| SceneDocumentError::Hydration {
        scene_id: scene_id.to_owned(),
        entity_id: entity_id.to_owned(),
        component_kind: component_kind.to_owned(),
        message: format!("expected albedo color `{raw_value}` to contain only hex digits"),
    })
}

fn channel_to_f32(value: u8) -> f32 {
    f32::from(value) / 255.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_scene::{
        ParticleAlignMode2dSceneDocument, ParticleBlendMode2dSceneDocument,
        ParticleForce2dSceneDocument, ParticleLight2dSceneDocument,
        ParticleLightMode2dSceneDocument, ParticleLineAnchor2dSceneDocument,
        ParticleMaterial2dSceneDocument, ParticleMotionStretch2dSceneDocument,
        ParticleShape2dSceneDocument, ParticleShapeChoice2dSceneDocument,
        ParticleShapeKeyframe2dSceneDocument, ParticleSimulationSpace2dSceneDocument,
        ParticleSpawnArea2dSceneDocument, ParticleVelocityMode2dSceneDocument,
        PluginComponentHydrationContext, SceneCommand, SceneDocument, SceneEntityDocument,
        SceneMetadataDocument, SceneVisual2dDocument,
    };
    use std::collections::BTreeMap;

    #[test]
    fn particle_hydrator_carries_emitter_domain_payload() {
        let hydrator = ParticleEmitter2dPluginComponentHydrator;
        let payload = ParticleEmitter2dDocument {
            entity_name: String::new(),
            render_layer: "fx.particles".to_owned(),
            attached_to: Some("ship".to_owned()),
            local_offset: SceneVec2Document { x: -12.0, y: 1.0 },
            local_direction_degrees: 180.0,
            spawn_area: Some(ParticleSpawnArea2dSceneDocument::Rect {
                size: SceneVec2Document { x: 120.0, y: 20.0 },
            }),
            active: false,
            spawn_rate: 90.0,
            max_particles: 64,
            particle_lifetime: 0.5,
            lifetime_jitter: 0.2,
            initial_speed: 120.0,
            speed_jitter: 8.0,
            spread_degrees: 45.0,
            inherit_parent_velocity: 0.75,
            velocity_mode: Some(ParticleVelocityMode2dSceneDocument::SourceInertial),
            simulation_space: Some(ParticleSimulationSpace2dSceneDocument::Source),
            initial_size: 2.0,
            final_size: 8.0,
            size_jitter: 0.4,
            color: Some("#FFFFFFFF".to_owned()),
            color_ramp: None,
            z_index: 6.0,
            shape: Some(ParticleShape2dSceneDocument::Circle { segments: 8 }),
            shape_choices: vec![
                ParticleShapeChoice2dSceneDocument {
                    weight: 2.0,
                    shape: ParticleShape2dSceneDocument::Circle { segments: 8 },
                },
                ParticleShapeChoice2dSceneDocument {
                    weight: 1.0,
                    shape: ParticleShape2dSceneDocument::Line { length: 14.0 },
                },
            ],
            shape_over_lifetime: vec![
                ParticleShapeKeyframe2dSceneDocument {
                    t: 0.0,
                    shape: ParticleShape2dSceneDocument::Quad,
                },
                ParticleShapeKeyframe2dSceneDocument {
                    t: 0.75,
                    shape: ParticleShape2dSceneDocument::Circle { segments: 12 },
                },
            ],
            line_anchor: Some(ParticleLineAnchor2dSceneDocument::Start),
            align: Some(ParticleAlignMode2dSceneDocument::Emitter),
            blend_mode: Some(ParticleBlendMode2dSceneDocument::Additive),
            motion_stretch: Some(ParticleMotionStretch2dSceneDocument {
                enabled: true,
                velocity_scale: 2.2,
                max_length: 96.0,
                shutter_seconds: 0.03,
                tail_alpha: 0.25,
                head_alpha: 0.9,
            }),
            material: Some(ParticleMaterial2dSceneDocument {
                lighting_mode: None,
                receives_light: true,
                light_response: 0.6,
                light_receiver: None,
            }),
            light: Some(ParticleLight2dSceneDocument {
                radius: 24.0,
                intensity: 0.35,
                mode: ParticleLightMode2dSceneDocument::Source,
                glow: false,
            }),
            emission_rate_curve: None,
            size_curve: None,
            alpha_curve: None,
            speed_curve: None,
            forces: vec![
                ParticleForce2dSceneDocument::Gravity {
                    acceleration: SceneVec2Document { x: 0.0, y: -480.0 },
                },
                ParticleForce2dSceneDocument::Drag { coefficient: 1.8 },
            ],
            post_fx: Vec::new(),
        };
        let entity = test_entity("emitter");
        let document = test_document(entity.clone());
        let mut commands = Vec::new();

        hydrator
            .hydrate_plugin_payload(PluginComponentHydrationContext {
                source_mod: "test-mod",
                document: &document,
                entity: &entity,
                entity_name: "test-emitter",
                component_index: 0,
                component_type: "amigo.vfx.particles-2d.ParticleEmitter2D",
                payload: &payload,
                commands: &mut commands,
            })
            .expect("particle hydrator should accept plugin payload");

        let command = plugin_payload::<ParticleEmitter2dSceneCommand>(&commands);
        assert_eq!(command.source_mod, "test-mod");
        assert_eq!(command.entity_name, "test-emitter");
        assert_eq!(command.render_layer, "fx.particles");
        assert_eq!(command.attached_to.as_deref(), Some("ship"));
        assert_eq!(command.local_offset, Vec2::new(-12.0, 1.0));
        assert!((command.local_direction_radians - std::f32::consts::PI).abs() < 0.001);
        assert_eq!(
            command.spawn_area,
            ParticleSpawnArea2dSceneCommand::Rect {
                size: Vec2::new(120.0, 20.0)
            }
        );
        assert!(!command.active);
        assert_eq!(command.spawn_rate, 90.0);
        assert_eq!(command.max_particles, 64);
        assert_eq!(
            command.velocity_mode,
            ParticleVelocityMode2dSceneCommand::SourceInertial
        );
        assert_eq!(
            command.simulation_space,
            ParticleSimulationSpace2dSceneCommand::Source
        );
        assert_eq!(
            command.shape,
            ParticleShape2dSceneCommand::Circle { segments: 8 }
        );
        assert_eq!(command.shape_choices.len(), 2);
        assert_eq!(
            command.shape_choices[1].shape,
            ParticleShape2dSceneCommand::Line { length: 14.0 }
        );
        assert_eq!(command.shape_over_lifetime.len(), 2);
        assert_eq!(command.line_anchor, ParticleLineAnchor2dSceneCommand::Start);
        assert_eq!(command.align, ParticleAlignMode2dSceneCommand::Emitter);
        assert_eq!(
            command.blend_mode,
            ParticleBlendMode2dSceneCommand::Additive
        );
        let stretch = command
            .motion_stretch
            .expect("motion stretch should be preserved");
        assert!(stretch.enabled);
        assert_eq!(stretch.velocity_scale, 2.2);
        assert_eq!(stretch.max_length, 96.0);
        assert!(command.material.receives_light);
        assert_eq!(command.material.light_response, 0.6);
        let light = command.light.expect("particle light should be preserved");
        assert_eq!(light.radius, 24.0);
        assert_eq!(light.intensity, 0.35);
        assert_eq!(light.mode, ParticleLightMode2dSceneCommand::Source);
        assert_eq!(command.forces.len(), 2);
        assert_eq!(
            command.forces[0],
            ParticleForce2dSceneCommand::Gravity {
                acceleration: Vec2::new(0.0, -480.0)
            }
        );
        assert_eq!(
            command.forces[1],
            ParticleForce2dSceneCommand::Drag { coefficient: 1.8 }
        );
    }

    fn plugin_payload<T: 'static>(commands: &[SceneCommand]) -> &T {
        assert_eq!(commands.len(), 1);
        match &commands[0] {
            SceneCommand::Plugin { command } => command
                .payload_as::<T>()
                .expect("plugin scene payload should downcast"),
            other => panic!("expected plugin scene command, got {other:?}"),
        }
    }

    fn test_entity(name: &str) -> SceneEntityDocument {
        SceneEntityDocument {
            id: name.to_owned(),
            name: name.to_owned(),
            tags: Vec::new(),
            groups: Vec::new(),
            visible: true,
            simulation_enabled: true,
            collision_enabled: true,
            properties: BTreeMap::new(),
            transform2: None,
            transform3: None,
            post_fx: Vec::new(),
            prefab: None,
            prefab_overrides: Vec::new(),
            components: Vec::new(),
        }
    }

    fn test_document(entity: SceneEntityDocument) -> SceneDocument {
        SceneDocument {
            version: 1,
            scene: SceneMetadataDocument {
                id: "test-scene".to_owned(),
                label: String::new(),
                description: None,
            },
            transitions: Vec::new(),
            collision_events: Vec::new(),
            audio_cues: Vec::new(),
            activation_sets: Vec::new(),
            visual2d: SceneVisual2dDocument::default(),
            state: BTreeMap::new(),
            entities: vec![entity],
        }
    }
}
