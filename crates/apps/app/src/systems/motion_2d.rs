use super::super::*;
use crate::runtime_context::RuntimeContext;
use amigo_2d_motion::{
    BoundsOutcome2d, Facing2d, MotionAnimationState, MotionState2d, apply_bounds_2d,
    drive_motion_2d, motion_animation_state_for, step_freeflight_motion_2d, step_velocity_2d,
};

pub(crate) fn tick_motion_2d_world(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let scene_service = ctx.required::<SceneService>()?;
    let physics_scene_service = ctx.required::<Physics2dSceneService>()?;
    let motion_scene_service = ctx.required::<Motion2dSceneService>()?;
    let script_event_queue = ctx.required::<ScriptEventQueue>()?;

    let static_colliders = physics_scene_service.static_colliders();
    let triggers = physics_scene_service.triggers();

    for body_command in physics_scene_service.kinematic_bodies() {
        let entity_name = body_command.entity_name.clone();
        if !scene_service.is_simulation_enabled(&entity_name) {
            continue;
        }
        let Some(mut transform) = scene_service.transform_of(&entity_name) else {
            continue;
        };
        let Some(collider_command) = physics_scene_service.aabb_collider(&entity_name) else {
            continue;
        };

        let mut body_state = physics_scene_service
            .body_state(&entity_name)
            .unwrap_or_default();
        let controller_command = motion_scene_service.motion_controller(&entity_name);
        let previous_controller_state = motion_scene_service
            .motion_state(&entity_name)
            .unwrap_or_else(|| MotionState2d {
                grounded: body_state.grounded.grounded,
                facing: Facing2d::Right,
                animation: MotionAnimationState::Idle,
                velocity: body_state.velocity,
            });

        let mut facing = previous_controller_state.facing;
        if let Some(controller_command) = controller_command.as_ref() {
            let motor = motion_scene_service
                .motion_intent(&entity_name)
                .unwrap_or_default();
            let drive = drive_motion_2d(
                &controller_command.controller.params,
                &body_state,
                &motor,
                facing,
                delta_seconds,
            );
            body_state.velocity = drive.velocity;
            facing = drive.facing;

            if drive.jumped {
                script_event_queue
                    .publish(ScriptEvent::new("player.jump", vec![entity_name.clone()]));
            }
        } else {
            body_state.velocity.y += -980.0 * body_command.body.gravity_scale * delta_seconds;
            if body_command.body.terminal_velocity > 0.0 {
                body_state.velocity.y = body_state
                    .velocity
                    .y
                    .max(-body_command.body.terminal_velocity.abs());
            }
        }

        let translation = Vec2::new(transform.translation.x, transform.translation.y);
        let step = move_and_collide(
            translation,
            &collider_command.collider,
            body_state.velocity,
            delta_seconds,
            &static_colliders,
        );

        body_state.velocity = step.velocity;
        body_state.grounded = step.grounded.clone();
        transform.translation.x = step.translation.x;
        transform.translation.y = step.translation.y;
        let _ = scene_service.set_transform(&entity_name, transform);
        let _ = physics_scene_service.sync_body_state(&entity_name, body_state.clone());

        if controller_command.is_some() {
            let _ = motion_scene_service.sync_motion_state(
                &entity_name,
                MotionState2d {
                    grounded: step.grounded.grounded,
                    facing,
                    animation: motion_animation_state_for(step.velocity, step.grounded.grounded),
                    velocity: step.velocity,
                },
            );
            motion_scene_service.clear_motion_intent(&entity_name);
        }

        for trigger in &triggers {
            let trigger_translation =
                scene_service
                    .transform_of(&trigger.entity_name)
                    .map(|transform| {
                        amigo_math::Vec2::new(
                            transform.translation.x + trigger.trigger.offset.x,
                            transform.translation.y + trigger.trigger.offset.y,
                        )
                    });
            let overlapping = overlaps_trigger_with_translation(
                step.translation,
                &collider_command.collider,
                trigger,
                trigger_translation,
            );
            let was_active =
                physics_scene_service.is_trigger_overlap_active(&trigger.entity_name, &entity_name);

            if overlapping && !was_active {
                physics_scene_service.set_trigger_overlap_active(
                    &trigger.entity_name,
                    &entity_name,
                    true,
                );
                if let Some(topic) = trigger.trigger.topic.as_ref() {
                    script_event_queue.publish(ScriptEvent::new(
                        topic.clone(),
                        vec![trigger.entity_name.clone(), entity_name.clone()],
                    ));
                }
            } else if !overlapping && was_active {
                physics_scene_service.set_trigger_overlap_active(
                    &trigger.entity_name,
                    &entity_name,
                    false,
                );
            }
        }
    }

    for command in motion_scene_service.freeflight_commands() {
        let entity_name = command.entity_name.clone();
        if !scene_service.is_simulation_enabled(&entity_name) {
            continue;
        }
        let Some(mut transform) = scene_service.transform_of(&entity_name) else {
            continue;
        };
        let state = motion_scene_service
            .freeflight_state(&entity_name)
            .unwrap_or(command.initial_state);
        let intent = motion_scene_service
            .freeflight_intent(&entity_name)
            .unwrap_or_default();
        let step = step_freeflight_motion_2d(&command.profile, &intent, state, delta_seconds);
        transform.translation.x += step.translation_delta.x;
        transform.translation.y += step.translation_delta.y;
        transform.rotation_euler.z += step.rotation_delta;
        let _ = scene_service.set_transform(&entity_name, transform);
        let _ = motion_scene_service.sync_freeflight_state(&entity_name, step.state);
        motion_scene_service.clear_freeflight_intent(&entity_name);
    }

    for command in motion_scene_service.velocities() {
        let entity_name = command.entity_name.clone();
        if !scene_service.is_simulation_enabled(&entity_name)
            || motion_scene_service
                .freeflight_command(&entity_name)
                .is_some()
        {
            continue;
        }
        let Some(mut transform) = scene_service.transform_of(&entity_name) else {
            continue;
        };
        let next = step_velocity_2d(
            Vec2::new(transform.translation.x, transform.translation.y),
            &command.velocity,
            delta_seconds,
        );
        transform.translation.x = next.x;
        transform.translation.y = next.y;
        let _ = scene_service.set_transform(&entity_name, transform);
    }

    for command in motion_scene_service.bounds() {
        let entity_name = command.entity_name.clone();
        if !scene_service.is_simulation_enabled(&entity_name) {
            continue;
        }
        let Some(mut transform) = scene_service.transform_of(&entity_name) else {
            continue;
        };
        let velocity = motion_scene_service.current_velocity(&entity_name);
        let result = apply_bounds_2d(
            Vec2::new(transform.translation.x, transform.translation.y),
            velocity,
            &command.bounds,
        );
        if matches!(result.outcome, BoundsOutcome2d::None) {
            continue;
        }

        match result.outcome {
            BoundsOutcome2d::None => {}
            BoundsOutcome2d::Hidden { .. } => {
                let _ = scene_service.set_visible(&entity_name, false);
                let _ = scene_service.set_simulation_enabled(&entity_name, false);
                let _ = scene_service.set_collision_enabled(&entity_name, false);
            }
            BoundsOutcome2d::Despawned { .. } => {
                let _ = scene_service.remove_entities_by_name(&[entity_name.clone()]);
            }
            BoundsOutcome2d::Bounced { .. }
            | BoundsOutcome2d::Wrapped { .. }
            | BoundsOutcome2d::Clamped { .. } => {
                transform.translation.x = result.translation.x;
                transform.translation.y = result.translation.y;
                let _ = scene_service.set_transform(&entity_name, transform);
            }
        }

        let _ = motion_scene_service.set_velocity(&entity_name, result.velocity);
        if let Some(mut state) = motion_scene_service.freeflight_state(&entity_name) {
            state.velocity = result.velocity;
            let _ = motion_scene_service.sync_freeflight_state(&entity_name, state);
        }
        if let Some(mut body_state) = physics_scene_service.body_state(&entity_name) {
            body_state.velocity = result.velocity;
            let _ = physics_scene_service.sync_body_state(&entity_name, body_state);
        }
    }

    Ok(())
}
