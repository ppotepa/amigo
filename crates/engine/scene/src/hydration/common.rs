use amigo_math::{Transform2, Transform3, Vec2, Vec3};

use super::events::curve1d_from_optional_document;
use super::particles::color_ramp_from_document;
use crate::*;

pub fn scene_key_from_document(document: &SceneDocument) -> SceneKey {
    SceneKey::new(document.scene.id.clone())
}

pub fn entity_selector_from_document(selector: &SceneEntitySelectorDocument) -> EntitySelector {
    match selector.kind {
        SceneEntitySelectorKindDocument::Entity => EntitySelector::Entity(selector.value.clone()),
        SceneEntitySelectorKindDocument::Tag => EntitySelector::Tag(selector.value.clone()),
        SceneEntitySelectorKindDocument::Group => EntitySelector::Group(selector.value.clone()),
        SceneEntitySelectorKindDocument::Pool => EntitySelector::Pool(selector.value.clone()),
    }
}

pub(super) fn input_action_binding_from_document(
    binding: &SceneInputActionBindingDocument,
) -> InputActionBindingSceneCommand {
    match binding {
        SceneInputActionBindingDocument::Axis { positive, negative } => {
            InputActionBindingSceneCommand::Axis {
                positive: positive.clone(),
                negative: negative.clone(),
            }
        }
        SceneInputActionBindingDocument::Button { pressed } => {
            InputActionBindingSceneCommand::Button {
                pressed: pressed.clone(),
            }
        }
    }
}

pub(super) fn behavior_from_document(
    behavior: &SceneBehaviorDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<BehaviorKindSceneCommand> {
    Ok(match behavior {
        SceneBehaviorDocument::FreeflightInputController { target, input } => {
            BehaviorKindSceneCommand::FreeflightInputController {
                target_entity: target.clone(),
                thrust_action: input.thrust.clone(),
                turn_action: input.turn.clone(),
                strafe_action: input.strafe.clone(),
            }
        }
        SceneBehaviorDocument::ParticleIntensityController { emitter, action } => {
            BehaviorKindSceneCommand::ParticleIntensityController {
                emitter: emitter.clone(),
                action: action.clone(),
            }
        }
        SceneBehaviorDocument::ParticleProfileController {
            emitter,
            action,
            max_hold_seconds,
            phases,
        } => BehaviorKindSceneCommand::ParticleProfileController {
            emitter: emitter.clone(),
            action: action.clone(),
            max_hold_seconds: *max_hold_seconds,
            phases: phases
                .iter()
                .map(|phase| {
                    particle_profile_phase_from_document(phase, scene_id, entity_id, component_kind)
                })
                .collect::<SceneDocumentResult<Vec<_>>>()?,
        },
        SceneBehaviorDocument::CameraFollowModeController {
            camera,
            action,
            target,
            lerp,
            lookahead_velocity_scale,
            lookahead_max_distance,
            sway_amount,
            sway_frequency,
        } => BehaviorKindSceneCommand::CameraFollowModeController {
            camera: camera.clone(),
            action: action.clone(),
            target: target.clone(),
            lerp: *lerp,
            lookahead_velocity_scale: *lookahead_velocity_scale,
            lookahead_max_distance: *lookahead_max_distance,
            sway_amount: *sway_amount,
            sway_frequency: *sway_frequency,
        },
        SceneBehaviorDocument::ProjectileFireController {
            emitter,
            source,
            action,
            cooldown,
            cooldown_id,
            audio,
        } => BehaviorKindSceneCommand::ProjectileFireController {
            emitter: emitter.clone(),
            source: source.clone(),
            action: action.clone(),
            cooldown_seconds: *cooldown,
            cooldown_id: cooldown_id.clone(),
            audio: audio.clone(),
        },
        SceneBehaviorDocument::MenuNavigationController {
            index_state,
            item_count,
            item_count_state,
            up_action,
            down_action,
            confirm_action,
            wrap,
            move_audio,
            confirm_audio,
            confirm_events,
            selected_color_prefix,
            selected_color,
            unselected_color,
        } => BehaviorKindSceneCommand::MenuNavigationController {
            index_state: index_state.clone(),
            item_count: *item_count,
            item_count_state: item_count_state.clone(),
            up_action: up_action.clone(),
            down_action: down_action.clone(),
            confirm_action: confirm_action.clone(),
            wrap: *wrap,
            move_audio: move_audio.clone(),
            confirm_audio: confirm_audio.clone(),
            confirm_events: confirm_events.clone(),
            selected_color_prefix: selected_color_prefix.clone(),
            selected_color: selected_color.clone(),
            unselected_color: unselected_color.clone(),
        },
        SceneBehaviorDocument::SceneTransitionController { action, scene }
        | SceneBehaviorDocument::SceneBackController { action, scene } => {
            BehaviorKindSceneCommand::SceneTransitionController {
                action: action.clone(),
                scene: scene.clone(),
            }
        }
        SceneBehaviorDocument::SceneAutoTransitionController { scene } => {
            BehaviorKindSceneCommand::SceneAutoTransitionController {
                scene: scene.clone(),
            }
        }
        SceneBehaviorDocument::SetStateOnActionController {
            action,
            key,
            value,
            audio,
        } => BehaviorKindSceneCommand::SetStateOnActionController {
            action: action.clone(),
            key: key.clone(),
            value: value.clone(),
            audio: audio.clone(),
        },
        SceneBehaviorDocument::ToggleStateController {
            action,
            key,
            default,
            audio,
        } => BehaviorKindSceneCommand::ToggleStateController {
            action: action.clone(),
            key: key.clone(),
            default: *default,
            audio: audio.clone(),
        },
        SceneBehaviorDocument::UiThemeSwitcher { bindings, cycle } => {
            BehaviorKindSceneCommand::UiThemeSwitcher {
                bindings: bindings.clone(),
                cycle_action: cycle.clone(),
            }
        }
    })
}

pub(super) fn particle_profile_phase_from_document(
    phase: &SceneParticleProfilePhaseDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<ParticleProfilePhaseSceneCommand> {
    Ok(ParticleProfilePhaseSceneCommand {
        id: phase.id.clone(),
        start_seconds: phase.start_seconds,
        end_seconds: phase.end_seconds,
        velocity_mode: phase
            .velocity_mode
            .map(particle_profile_velocity_mode_from_document),
        color_ramp: phase
            .color_ramp
            .as_ref()
            .map(|ramp| color_ramp_from_document(ramp, scene_id, entity_id, component_kind))
            .transpose()?,
        spawn_rate: phase
            .spawn_rate
            .as_ref()
            .map(particle_profile_scalar_from_document),
        lifetime: phase
            .lifetime
            .as_ref()
            .map(particle_profile_scalar_from_document),
        lifetime_jitter: phase
            .lifetime_jitter
            .as_ref()
            .map(particle_profile_scalar_from_document),
        speed: phase
            .speed
            .as_ref()
            .map(particle_profile_scalar_from_document),
        speed_jitter: phase
            .speed_jitter
            .as_ref()
            .map(particle_profile_scalar_from_document),
        spread_degrees: phase
            .spread_degrees
            .as_ref()
            .map(particle_profile_scalar_from_document),
        initial_size: phase
            .initial_size
            .as_ref()
            .map(particle_profile_scalar_from_document),
        final_size: phase
            .final_size
            .as_ref()
            .map(particle_profile_scalar_from_document),
        spawn_area_line: phase
            .spawn_area_line
            .as_ref()
            .map(particle_profile_scalar_from_document),
        shape_line: phase
            .shape_line
            .as_ref()
            .map(particle_profile_scalar_from_document),
        shape_circle_weight: phase
            .shape_circle_weight
            .as_ref()
            .map(particle_profile_scalar_from_document),
        shape_line_weight: phase
            .shape_line_weight
            .as_ref()
            .map(particle_profile_scalar_from_document),
        shape_quad_weight: phase
            .shape_quad_weight
            .as_ref()
            .map(particle_profile_scalar_from_document),
        size_curve: phase
            .size_curve
            .as_ref()
            .map(particle_profile_curve4_from_document),
        speed_curve: phase
            .speed_curve
            .as_ref()
            .map(particle_profile_curve4_from_document),
        alpha_curve: phase
            .alpha_curve
            .as_ref()
            .map(particle_profile_curve4_from_document),
        burst: phase
            .burst
            .as_ref()
            .map(particle_profile_burst_from_document),
        clear_forces: phase.clear_forces,
    })
}

pub(super) fn particle_profile_velocity_mode_from_document(
    document: SceneParticleProfileVelocityModeDocument,
) -> ParticleProfileVelocityModeSceneCommand {
    match document {
        SceneParticleProfileVelocityModeDocument::Free => {
            ParticleProfileVelocityModeSceneCommand::Free
        }
        SceneParticleProfileVelocityModeDocument::SourceInertial => {
            ParticleProfileVelocityModeSceneCommand::SourceInertial
        }
    }
}

pub(super) fn particle_profile_scalar_from_document(
    document: &SceneParticleProfileScalarDocument,
) -> ParticleProfileScalarSceneCommand {
    ParticleProfileScalarSceneCommand {
        from: document.from,
        to: document.to,
        curve: curve1d_from_optional_document(document.curve.as_ref()),
        intensity_scale: document.intensity_scale,
        noise_scale: document.noise_scale,
    }
}

pub(super) fn particle_profile_curve4_from_document(
    document: &SceneParticleProfileCurve4Document,
) -> ParticleProfileCurve4SceneCommand {
    ParticleProfileCurve4SceneCommand {
        v0: particle_profile_scalar_from_document(&document.v0),
        v1: particle_profile_scalar_from_document(&document.v1),
        v2: particle_profile_scalar_from_document(&document.v2),
        v3: particle_profile_scalar_from_document(&document.v3),
    }
}

pub(super) fn particle_profile_burst_from_document(
    document: &SceneParticleProfileBurstDocument,
) -> ParticleProfileBurstSceneCommand {
    ParticleProfileBurstSceneCommand {
        rate_hz: document.rate_hz,
        min_count: document.min_count,
        max_count: document.max_count,
        threshold: document.threshold,
    }
}

impl From<SceneEntitySelectorDocument> for EntitySelector {
    fn from(selector: SceneEntitySelectorDocument) -> Self {
        match selector.kind {
            SceneEntitySelectorKindDocument::Entity => Self::Entity(selector.value),
            SceneEntitySelectorKindDocument::Tag => Self::Tag(selector.value),
            SceneEntitySelectorKindDocument::Group => Self::Group(selector.value),
            SceneEntitySelectorKindDocument::Pool => Self::Pool(selector.value),
        }
    }
}

pub(super) fn transform2_for_entity(entity: &SceneEntityDocument) -> Transform2 {
    entity
        .transform2
        .map(transform2_from_document)
        .or_else(|| entity.transform3.map(transform2_from_transform3_document))
        .unwrap_or_default()
}

pub(super) fn transform3_for_entity(entity: &SceneEntityDocument) -> Transform3 {
    entity
        .transform3
        .map(transform3_from_document)
        .or_else(|| entity.transform2.map(transform3_from_transform2_document))
        .unwrap_or_default()
}

pub(super) fn lifecycle_for_entity(entity: &SceneEntityDocument) -> SceneEntityLifecycle {
    SceneEntityLifecycle {
        visible: entity.visible,
        simulation_enabled: entity.simulation_enabled,
        collision_enabled: entity.collision_enabled,
    }
}

pub(super) fn property_value_from_document(value: &ScenePropertyValueDocument) -> ScenePropertyValue {
    match value {
        ScenePropertyValueDocument::Bool(value) => ScenePropertyValue::Bool(*value),
        ScenePropertyValueDocument::Int(value) => ScenePropertyValue::Int(*value),
        ScenePropertyValueDocument::Float(value) => ScenePropertyValue::Float(*value),
        ScenePropertyValueDocument::String(value) => ScenePropertyValue::String(value.clone()),
    }
}

pub(super) fn resolve_scene_audio_clip(source_mod: &str, clip: &str) -> String {
    if clip.contains('/') {
        clip.to_owned()
    } else {
        format!("{source_mod}/audio/{clip}")
    }
}

pub(super) fn lifetime_outcome_from_document(
    outcome: SceneLifetimeExpirationOutcomeDocument,
    pool: Option<String>,
) -> LifetimeExpirationOutcome {
    match outcome {
        SceneLifetimeExpirationOutcomeDocument::Hide => LifetimeExpirationOutcome::Hide,
        SceneLifetimeExpirationOutcomeDocument::Disable => LifetimeExpirationOutcome::Disable,
        SceneLifetimeExpirationOutcomeDocument::Despawn => LifetimeExpirationOutcome::Despawn,
        SceneLifetimeExpirationOutcomeDocument::ReturnToPool => {
            LifetimeExpirationOutcome::ReturnToPool {
                pool: pool.unwrap_or_default(),
            }
        }
    }
}

pub(super) fn bounds_behavior_from_document(
    behavior: SceneBoundsBehavior2dDocument,
    restitution: f32,
) -> BoundsBehavior2dSceneCommand {
    match behavior {
        SceneBoundsBehavior2dDocument::Bounce => BoundsBehavior2dSceneCommand::Bounce {
            restitution: restitution.max(0.0),
        },
        SceneBoundsBehavior2dDocument::Wrap => BoundsBehavior2dSceneCommand::Wrap,
        SceneBoundsBehavior2dDocument::Hide => BoundsBehavior2dSceneCommand::Hide,
        SceneBoundsBehavior2dDocument::Despawn => BoundsBehavior2dSceneCommand::Despawn,
        SceneBoundsBehavior2dDocument::Clamp => BoundsBehavior2dSceneCommand::Clamp,
    }
}

pub(super) fn transform2_from_document(document: SceneTransform2Document) -> Transform2 {
    Transform2 {
        translation: vec2_from_document(document.translation),
        rotation_radians: document.rotation_radians,
        scale: vec2_from_document(document.scale),
    }
}

pub(super) fn transform3_from_document(document: SceneTransform3Document) -> Transform3 {
    Transform3 {
        translation: vec3_from_document(document.translation),
        rotation_euler: vec3_from_document(document.rotation_euler),
        scale: vec3_from_document(document.scale),
    }
}

pub(super) fn transform3_from_transform2_document(document: SceneTransform2Document) -> Transform3 {
    Transform3 {
        translation: Vec3::new(document.translation.x, document.translation.y, 0.0),
        rotation_euler: Vec3::new(0.0, 0.0, document.rotation_radians),
        scale: Vec3::new(document.scale.x, document.scale.y, 1.0),
    }
}

pub(super) fn transform2_from_transform3_document(document: SceneTransform3Document) -> Transform2 {
    Transform2 {
        translation: Vec2::new(document.translation.x, document.translation.y),
        rotation_radians: document.rotation_euler.z,
        scale: Vec2::new(document.scale.x, document.scale.y),
    }
}

pub(super) fn vec2_from_document(value: crate::SceneVec2Document) -> Vec2 {
    Vec2::new(value.x, value.y)
}

pub(super) fn vec3_from_document(value: crate::SceneVec3Document) -> Vec3 {
    Vec3::new(value.x, value.y, value.z)
}

pub(super) fn sprite_sheet_from_document(value: SceneSpriteSheetDocument) -> SpriteSheet2dSceneCommand {
    SpriteSheet2dSceneCommand {
        columns: value.columns.max(1),
        rows: value.rows.max(1),
        frame_count: value.frame_count.max(1),
        frame_size: vec2_from_document(value.frame_size),
        fps: value.fps.max(0.0),
        looping: value.looping,
    }
}

pub(super) fn sprite_animation_from_document(
    value: crate::SceneSpriteAnimationDocument,
) -> SpriteAnimation2dSceneOverride {
    SpriteAnimation2dSceneOverride {
        fps: value.fps.map(|fps| fps.max(0.0)),
        looping: value.looping,
        start_frame: value.start_frame,
    }
}

