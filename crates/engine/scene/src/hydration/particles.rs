use amigo_fx::{ColorInterpolation, ColorRamp, ColorStop};

use super::common::vec2_from_document;
use crate::*;
use super::style::parse_color_rgba_hex;

pub(super) fn color_ramp_from_document(
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

pub(super) fn particle_shape_from_document(
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

pub(super) fn particle_line_anchor_from_document(
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

pub(super) fn particle_align_from_document(
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

pub(super) fn particle_blend_from_document(
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

pub(super) fn particle_velocity_mode_from_document(
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

pub(super) fn particle_simulation_space_from_document(
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

pub(super) fn particle_light_mode_from_document(
    document: ParticleLightMode2dSceneDocument,
) -> crate::ParticleLightMode2dSceneCommand {
    match document {
        ParticleLightMode2dSceneDocument::Source => crate::ParticleLightMode2dSceneCommand::Source,
        ParticleLightMode2dSceneDocument::Particle => {
            crate::ParticleLightMode2dSceneCommand::Particle
        }
    }
}

pub(super) fn particle_spawn_area_from_document(
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

pub(super) fn particle_force_from_document(
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

