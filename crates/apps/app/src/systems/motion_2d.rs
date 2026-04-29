use super::super::*;
use crate::runtime_context::RuntimeContext;

pub(crate) fn tick_motion_2d_world(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let scene_service = ctx.required::<SceneService>()?;
    let physics_scene_service = ctx.required::<Physics2dSceneService>()?;
    let motion_scene_service = ctx.required::<PlatformerSceneService>()?;
    let script_event_queue = ctx.required::<ScriptEventQueue>()?;

    let static_colliders = physics_scene_service.static_colliders();
    let triggers = physics_scene_service.triggers();

    for body_command in physics_scene_service.kinematic_bodies() {
        let entity_name = body_command.entity_name.clone();
        let Some(mut transform) = scene_service.transform_of(&entity_name) else {
            continue;
        };
        let Some(collider_command) = physics_scene_service.aabb_collider(&entity_name) else {
            continue;
        };

        let mut body_state = physics_scene_service
            .body_state(&entity_name)
            .unwrap_or_default();
        let controller_command = motion_scene_service.controller(&entity_name);
        let previous_controller_state = motion_scene_service
            .state(&entity_name)
            .unwrap_or_else(|| PlatformerControllerState {
                grounded: body_state.grounded.grounded,
                facing: PlatformerFacing::Right,
                animation: PlatformerAnimationState::Idle,
                velocity: body_state.velocity,
            });

        let mut facing = previous_controller_state.facing;
        if let Some(controller_command) = controller_command.as_ref() {
            let motor = motion_scene_service
                .motor(&entity_name)
                .unwrap_or_default();
            let drive = drive_controller(
                &controller_command.controller.params,
                &body_state,
                &motor,
                facing,
                delta_seconds,
            );
            body_state.velocity = drive.velocity;
            facing = drive.facing;

            if drive.jumped {
                script_event_queue.publish(ScriptEvent::new("player.jump", vec![entity_name.clone()]));
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
            let _ = motion_scene_service.sync_state(
                &entity_name,
                PlatformerControllerState {
                    grounded: step.grounded.grounded,
                    facing,
                    animation: animation_state_for(step.velocity, step.grounded.grounded),
                    velocity: step.velocity,
                },
            );
            motion_scene_service.clear_motor(&entity_name);
        }

        for trigger in &triggers {
            let trigger_translation = scene_service
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

    Ok(())
}

#[allow(dead_code)]
pub(crate) fn tick_platformer_world(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    tick_motion_2d_world(runtime, delta_seconds)
}
