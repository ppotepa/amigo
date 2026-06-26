use std::sync::Arc;

use amigo_core::{AmigoError, AmigoResult};
use amigo_input_actions::InputActionService;
use amigo_input_api::{InputState, KeyCode, MouseButton};
use amigo_math::{Transform3, Vec2, Vec3};
use amigo_runtime::Runtime;
use amigo_scene::{CameraController3dModeSceneCommand, SceneService};

use crate::{
    CameraController3dRuntimeState, CameraController3dSceneService, CameraFollow2dSceneService,
    CameraService, Parallax2dSceneService,
};

fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> AmigoResult<Arc<T>> {
    runtime.resolve::<T>().ok_or_else(|| {
        AmigoError::Message(format!(
            "required service `{}` is not registered",
            std::any::type_name::<T>()
        ))
    })
}

pub fn tick_camera_follow_2d_system(runtime: &Runtime) -> AmigoResult<()> {
    tick_camera_follow_world(runtime, amigo_session::simulation_delta_seconds(runtime))
}

pub fn tick_camera_controller_3d_system(runtime: &Runtime) -> AmigoResult<()> {
    let scene_service = required::<SceneService>(runtime)?;
    let controller_service = required::<CameraController3dSceneService>(runtime)?;
    let Some(input) = runtime.resolve::<InputState>() else {
        return Ok(());
    };
    let actions = runtime.resolve::<InputActionService>();
    let delta_seconds = amigo_session::simulation_delta_seconds(runtime).max(0.0);

    for controller in controller_service.controllers() {
        let mut next = controller.clone();
        if action_pressed(
            &input,
            actions.as_deref(),
            next.command.switch_action.as_deref(),
        ) {
            next.mode = match next.mode {
                CameraController3dModeSceneCommand::Orbit => {
                    CameraController3dModeSceneCommand::Freelook
                }
                CameraController3dModeSceneCommand::Freelook => {
                    CameraController3dModeSceneCommand::Orbit
                }
            };
            next.last_cursor = None;
        }

        let cursor = input.cursor_position();
        let cursor_delta = cursor_delta(next.last_cursor, cursor);
        next.last_cursor = cursor;

        match next.mode {
            CameraController3dModeSceneCommand::Orbit => {
                apply_orbit_controller(&scene_service, &input, cursor_delta, &mut next);
            }
            CameraController3dModeSceneCommand::Freelook => {
                apply_freelook_controller(
                    &scene_service,
                    &input,
                    actions.as_deref(),
                    cursor_delta,
                    delta_seconds,
                    &mut next,
                );
            }
        }

        let entity_name = next.command.entity_name.clone();
        let _ = controller_service.update_controller(&entity_name, |state| {
            *state = next;
        });
    }

    Ok(())
}

pub fn tick_camera_follow_world(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let scene_service = required::<SceneService>(runtime)?;
    let camera_follow_scene_service = required::<CameraFollow2dSceneService>(runtime)?;

    for follow in camera_follow_scene_service.commands() {
        let Some(target_transform) = scene_service.transform_of(&follow.target) else {
            continue;
        };
        let Some(mut camera_transform) = scene_service.transform_of(&follow.entity_name) else {
            continue;
        };

        let velocity = Vec2::ZERO;
        let speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
        let (velocity_dir_x, velocity_dir_y) = if speed > f32::EPSILON {
            (velocity.x / speed, velocity.y / speed)
        } else {
            (0.0, 0.0)
        };
        let lookahead_distance =
            (speed * follow.lookahead_velocity_scale).min(follow.lookahead_max_distance.max(0.0));
        let speed_factor = (speed / 360.0).clamp(0.0, 1.0);
        let sway_phase = (target_transform.translation.x * 0.013
            + target_transform.translation.y * 0.017)
            * follow.sway_frequency.max(0.0);
        let sway = sway_phase.sin() * follow.sway_amount * speed_factor;
        let perpendicular_x = -velocity_dir_y;
        let perpendicular_y = velocity_dir_x;

        let desired_x = target_transform.translation.x
            + follow.offset.x
            + velocity_dir_x * lookahead_distance
            + perpendicular_x * sway;
        let desired_y = target_transform.translation.y
            + follow.offset.y
            + velocity_dir_y * lookahead_distance
            + perpendicular_y * sway;
        let alpha = if follow.lerp >= 1.0 {
            1.0
        } else {
            1.0 - (1.0 - follow.lerp.clamp(0.0, 1.0)).powf((delta_seconds * 60.0).max(0.0))
        };

        if alpha >= 1.0 {
            camera_transform.translation.x = desired_x;
            camera_transform.translation.y = desired_y;
        } else {
            camera_transform.translation.x += (desired_x - camera_transform.translation.x) * alpha;
            camera_transform.translation.y += (desired_y - camera_transform.translation.y) * alpha;
        }

        let _ = scene_service.set_transform(&follow.entity_name, camera_transform);
    }

    Ok(())
}

fn action_pressed(
    input: &InputState,
    actions: Option<&InputActionService>,
    action: Option<&str>,
) -> bool {
    let Some(action) = action else {
        return input.was_pressed(KeyCode::F);
    };
    actions
        .map(|actions| actions.pressed(input, action))
        .unwrap_or(false)
}

fn action_axis(input: &InputState, actions: Option<&InputActionService>, action: &str) -> f32 {
    actions
        .map(|actions| actions.axis(input, action))
        .unwrap_or(0.0)
}

fn cursor_delta(previous: Option<(f32, f32)>, current: Option<(f32, f32)>) -> Vec2 {
    match (previous, current) {
        (Some(previous), Some(current)) => {
            Vec2::new(current.0 - previous.0, current.1 - previous.1)
        }
        _ => Vec2::ZERO,
    }
}

fn apply_orbit_controller(
    scene: &SceneService,
    input: &InputState,
    cursor_delta: Vec2,
    state: &mut CameraController3dRuntimeState,
) {
    if input.is_mouse_down(MouseButton::Left) {
        state.yaw -= cursor_delta.x * state.command.orbit_sensitivity.max(0.0);
        state.pitch = (state.pitch - cursor_delta.y * state.command.orbit_sensitivity.max(0.0))
            .clamp(-1.45, 1.45);
    }

    let min_distance = state.command.orbit_min_distance.max(0.01);
    let max_distance = state.command.orbit_max_distance.max(min_distance);
    state.distance = (state.distance
        * zoom_factor_from_wheel(
            normalized_wheel_steps(input),
            state.command.orbit_zoom_speed,
        ))
    .clamp(min_distance, max_distance);

    let target = state
        .command
        .orbit_target
        .as_deref()
        .and_then(|target| scene.transform_of(target))
        .map(|transform| transform.translation)
        .unwrap_or(Vec3::ZERO);

    let cos_pitch = state.pitch.cos();
    let offset = Vec3::new(
        state.yaw.sin() * cos_pitch * state.distance,
        state.pitch.sin() * state.distance,
        state.yaw.cos() * cos_pitch * state.distance,
    );
    let transform = Transform3 {
        translation: add_vec3(target, offset),
        rotation_euler: Vec3::new(-state.pitch, state.yaw, 0.0),
        scale: Vec3::ONE,
    };
    let _ = scene.set_transform(&state.command.camera, transform);
}

fn apply_freelook_controller(
    scene: &SceneService,
    input: &InputState,
    actions: Option<&InputActionService>,
    cursor_delta: Vec2,
    delta_seconds: f32,
    state: &mut CameraController3dRuntimeState,
) {
    let Some(mut transform) = scene.transform_of(&state.command.camera) else {
        return;
    };

    if input.is_mouse_down(MouseButton::Right) || input.is_mouse_down(MouseButton::Left) {
        state.yaw -= cursor_delta.x * state.command.freelook_sensitivity.max(0.0);
        state.pitch = (state.pitch - cursor_delta.y * state.command.freelook_sensitivity.max(0.0))
            .clamp(-1.45, 1.45);
        transform.rotation_euler = Vec3::new(-state.pitch, state.yaw, 0.0);
    }

    let forward_axis = action_axis(input, actions, &state.command.move_forward_action);
    let strafe_axis = action_axis(input, actions, &state.command.move_strafe_action);
    let lift_axis = action_axis(input, actions, &state.command.move_lift_action);
    let wheel_steps = normalized_wheel_steps(input);
    if wheel_steps.abs() > f32::EPSILON {
        state.freelook_speed_multiplier = (state.freelook_speed_multiplier
            * zoom_factor_from_wheel(-wheel_steps, state.command.orbit_zoom_speed))
        .clamp(0.15, 6.0);
    }
    let speed =
        state.command.freelook_speed.max(0.0) * state.freelook_speed_multiplier * delta_seconds;
    let forward = Vec3::new(-state.yaw.sin(), state.pitch.sin(), -state.yaw.cos());
    let right = Vec3::new(state.yaw.cos(), 0.0, -state.yaw.sin());
    let up = Vec3::new(0.0, 1.0, 0.0);
    transform.translation = add_vec3(
        transform.translation,
        scale_vec3(forward, forward_axis * speed),
    );
    transform.translation = add_vec3(
        transform.translation,
        scale_vec3(right, strafe_axis * speed),
    );
    transform.translation = add_vec3(transform.translation, scale_vec3(up, lift_axis * speed));

    let _ = scene.set_transform(&state.command.camera, transform);
}

fn add_vec3(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(left.x + right.x, left.y + right.y, left.z + right.z)
}

fn scale_vec3(value: Vec3, scale: f32) -> Vec3 {
    Vec3::new(value.x * scale, value.y * scale, value.z * scale)
}

fn normalized_wheel_steps(input: &InputState) -> f32 {
    let delta = input.mouse_wheel_delta_y();
    if delta.is_finite() {
        delta.clamp(-3.0, 3.0)
    } else {
        0.0
    }
}

fn zoom_factor_from_wheel(steps: f32, speed: f32) -> f32 {
    (1.0 - steps * speed.max(0.0)).clamp(0.84, 1.19)
}

pub fn tick_parallax_2d_system(runtime: &Runtime) -> AmigoResult<()> {
    tick_parallax_world(runtime)
}

pub fn tick_camera_focus_transition_2d_system(runtime: &Runtime) -> AmigoResult<()> {
    let Some(camera_service) = runtime.resolve::<CameraService>() else {
        return Ok(());
    };
    let delta_seconds = amigo_session::simulation_delta_seconds(runtime);
    camera_service.tick_focus_transitions_2d(delta_seconds);
    camera_service.tick_sway_2d(delta_seconds);
    Ok(())
}

pub fn tick_parallax_world(runtime: &Runtime) -> AmigoResult<()> {
    let scene_service = required::<SceneService>(runtime)?;
    let parallax_scene_service = required::<Parallax2dSceneService>(runtime)?;

    for parallax in parallax_scene_service.commands() {
        let Some(camera_transform) = scene_service.transform_of(&parallax.camera) else {
            continue;
        };
        let Some(mut entity_transform) = scene_service.transform_of(&parallax.entity_name) else {
            continue;
        };

        let camera_translation = Vec2::new(
            camera_transform.translation.x,
            camera_transform.translation.y,
        );
        let camera_origin = parallax.camera_origin.unwrap_or(camera_translation);
        if parallax.camera_origin.is_none() {
            let _ =
                parallax_scene_service.set_camera_origin(&parallax.entity_name, camera_translation);
        }

        let factor_x = parallax.factor.x.clamp(0.0, 1.0);
        let factor_y = parallax.factor.y.clamp(0.0, 1.0);
        entity_transform.translation.x =
            parallax.anchor.x + (camera_translation.x - camera_origin.x) * (1.0 - factor_x);
        entity_transform.translation.y =
            parallax.anchor.y + (camera_translation.y - camera_origin.y) * (1.0 - factor_y);

        let _ = scene_service.set_transform(&parallax.entity_name, entity_transform);
    }

    Ok(())
}
