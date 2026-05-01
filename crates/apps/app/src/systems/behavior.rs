use amigo_2d_motion::{FreeflightMotionIntent2d, Motion2dSceneService, projectile_launch_2d};
use amigo_2d_particles::Particle2dSceneService;
use amigo_2d_physics::Physics2dSceneService;
use amigo_audio_api::{AudioClipKey, AudioCommand, AudioCommandQueue};
use amigo_behavior::{BehaviorKind, BehaviorSceneService};
use amigo_core::AmigoResult;
use amigo_input_actions::InputActionService;
use amigo_input_api::InputState;
use amigo_runtime::Runtime;
use amigo_scene::{
    CameraFollow2dSceneCommand, CameraFollow2dSceneService, EntityPoolSceneService,
    LifetimeSceneService, SceneCommand, SceneCommandQueue, SceneKey, SceneService,
};
use amigo_scripting_api::{ScriptEvent, ScriptEventQueue};
use amigo_state::SceneStateService;
use amigo_state::SceneTimerService;
use amigo_ui::UiThemeService;

use crate::runtime_context::required;

pub(crate) fn tick_behaviors(runtime: &Runtime) -> AmigoResult<()> {
    let behaviors = required::<BehaviorSceneService>(runtime)?;
    let actions = required::<InputActionService>(runtime)?;
    let input = required::<InputState>(runtime)?;
    let motion = required::<Motion2dSceneService>(runtime)?;
    let scene = required::<SceneService>(runtime)?;
    let pools = required::<EntityPoolSceneService>(runtime)?;
    let particles = runtime.resolve::<Particle2dSceneService>();
    let physics = runtime.resolve::<Physics2dSceneService>();
    let lifetimes = runtime.resolve::<LifetimeSceneService>();
    let timers = runtime.resolve::<SceneTimerService>();
    let audio = runtime.resolve::<AudioCommandQueue>();
    let scene_commands = runtime.resolve::<SceneCommandQueue>();
    let camera_follow = runtime.resolve::<CameraFollow2dSceneService>();
    let script_events = runtime.resolve::<ScriptEventQueue>();
    let scene_state = runtime.resolve::<SceneStateService>();
    let ui_theme = runtime.resolve::<UiThemeService>();

    for command in behaviors.behaviors() {
        if !behavior_condition_matches(command.condition.as_ref(), scene_state.as_deref()) {
            continue;
        }

        match command.behavior {
            BehaviorKind::FreeflightInputController(config) => {
                let thrust = actions.axis(input.as_ref(), &config.thrust_action);
                let turn = actions.axis(input.as_ref(), &config.turn_action);
                let strafe = config
                    .strafe_action
                    .as_deref()
                    .map(|action| actions.axis(input.as_ref(), action))
                    .unwrap_or(0.0);

                motion.drive_freeflight(
                    &config.target_entity,
                    FreeflightMotionIntent2d {
                        thrust,
                        strafe,
                        turn,
                    },
                );

                if let (Some(particles), Some(thruster)) =
                    (particles.as_ref(), config.thruster_emitter.as_ref())
                {
                    let intensity = thrust.abs().clamp(0.0, 1.0);
                    particles.set_active(thruster, intensity > 0.01);
                    particles.set_intensity(thruster, intensity);
                }
            }
            BehaviorKind::ParticleIntensityController(config) => {
                if let Some(particles) = particles.as_ref() {
                    let intensity = actions.axis(input.as_ref(), &config.action).abs();
                    particles.set_active(&config.emitter, intensity > 0.01);
                    particles.set_intensity(&config.emitter, intensity);
                }
            }
            BehaviorKind::CameraFollowModeController(config) => {
                if actions.pressed(input.as_ref(), &config.action) {
                    if let Some(camera_follow) = camera_follow.as_ref() {
                        apply_camera_follow_mode(camera_follow.as_ref(), &config);
                    }
                }
            }
            BehaviorKind::ProjectileFireController(config) => {
                if actions.pressed(input.as_ref(), &config.action) {
                    let cooldown_id = config
                        .cooldown_id
                        .clone()
                        .unwrap_or_else(|| format!("behavior.{}.cooldown", command.entity_name));

                    if timers
                        .as_ref()
                        .is_some_and(|timers| timers.active(&cooldown_id))
                    {
                        continue;
                    }

                    if fire_projectile_from_behavior(
                        scene.as_ref(),
                        motion.as_ref(),
                        pools.as_ref(),
                        physics.as_deref(),
                        lifetimes.as_deref(),
                        &config.emitter,
                        config.source.as_deref().unwrap_or(&config.emitter),
                    ) {
                        if config.cooldown_seconds.is_finite() && config.cooldown_seconds > 0.0 {
                            if let Some(timers) = timers.as_ref() {
                                let _ = timers.start(cooldown_id, config.cooldown_seconds);
                            }
                        }
                        if let (Some(audio), Some(clip)) = (audio.as_ref(), config.audio.as_ref()) {
                            audio.push(AudioCommand::PlayOnce {
                                clip: AudioClipKey::new(clip.clone()),
                            });
                        }
                    }
                }
            }
            BehaviorKind::MenuNavigationController(config) => {
                if let Some(scene_state) = scene_state.as_ref() {
                    tick_menu_navigation_controller(
                        actions.as_ref(),
                        input.as_ref(),
                        scene_state.as_ref(),
                        audio.as_deref(),
                        script_events.as_deref(),
                        &config,
                    );
                }
            }
            BehaviorKind::SceneTransitionController(config) => {
                if actions.pressed(input.as_ref(), &config.action) {
                    if let Some(scene_commands) = scene_commands.as_ref() {
                        scene_commands.submit(SceneCommand::SelectScene {
                            scene: SceneKey::new(config.scene),
                        });
                    }
                }
            }
            BehaviorKind::SceneAutoTransitionController(config) => {
                if let Some(scene_commands) = scene_commands.as_ref() {
                    scene_commands.submit(SceneCommand::SelectScene {
                        scene: SceneKey::new(config.scene),
                    });
                }
            }
            BehaviorKind::SetStateOnActionController(config) => {
                if actions.pressed(input.as_ref(), &config.action) {
                    if let Some(scene_state) = scene_state.as_ref() {
                        set_scene_state_from_string(
                            scene_state.as_ref(),
                            config.key.clone(),
                            config.value.clone(),
                        );
                    }
                    if let (Some(audio), Some(clip)) = (audio.as_ref(), config.audio.as_ref()) {
                        audio.push(AudioCommand::PlayOnce {
                            clip: AudioClipKey::new(clip.clone()),
                        });
                    }
                }
            }
            BehaviorKind::ToggleStateController(config) => {
                if actions.pressed(input.as_ref(), &config.action) {
                    if let Some(scene_state) = scene_state.as_ref() {
                        let next = !scene_state.get_bool(&config.key).unwrap_or(config.default);
                        scene_state.set_bool(&config.key, next);
                    }
                    if let (Some(audio), Some(clip)) = (audio.as_ref(), config.audio.as_ref()) {
                        audio.push(AudioCommand::PlayOnce {
                            clip: AudioClipKey::new(clip.clone()),
                        });
                    }
                }
            }
            BehaviorKind::UiThemeSwitcher(config) => {
                if let Some(ui_theme) = ui_theme.as_ref() {
                    for (action, theme_id) in &config.bindings {
                        if actions.pressed(input.as_ref(), action) {
                            ui_theme.set_active_theme(theme_id);
                        }
                    }

                    if let Some(cycle_action) = config.cycle_action.as_deref() {
                        if actions.pressed(input.as_ref(), cycle_action) {
                            cycle_theme(ui_theme.as_ref());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn tick_menu_navigation_controller(
    actions: &InputActionService,
    input: &InputState,
    scene_state: &SceneStateService,
    audio: Option<&AudioCommandQueue>,
    script_events: Option<&ScriptEventQueue>,
    config: &amigo_behavior::MenuNavigationControllerBehavior,
) {
    let item_count = config
        .item_count_state
        .as_deref()
        .and_then(|key| scene_state.get_int(key))
        .unwrap_or(config.item_count)
        .max(0);
    if item_count == 0 {
        return;
    }

    let current = scene_state
        .get_int(&config.index_state)
        .unwrap_or(0)
        .clamp(0, item_count - 1);
    let mut next = current;

    if actions.pressed(input, &config.up_action) {
        next -= 1;
        if next < 0 {
            next = if config.wrap { item_count - 1 } else { 0 };
        }
    }

    if actions.pressed(input, &config.down_action) {
        next += 1;
        if next >= item_count {
            next = if config.wrap { 0 } else { item_count - 1 };
        }
    }

    if next != current {
        scene_state.set_int(&config.index_state, next);
        if let (Some(audio), Some(clip)) = (audio, config.move_audio.as_ref()) {
            audio.push(AudioCommand::PlayOnce {
                clip: AudioClipKey::new(clip.clone()),
            });
        }
    }

    write_menu_selection_state(scene_state, config, next, item_count);

    let Some(confirm_action) = config.confirm_action.as_deref() else {
        return;
    };
    if !actions.pressed(input, confirm_action) {
        return;
    }

    if let (Some(audio), Some(clip)) = (audio, config.confirm_audio.as_ref()) {
        audio.push(AudioCommand::PlayOnce {
            clip: AudioClipKey::new(clip.clone()),
        });
    }

    let Some(topic) = usize::try_from(next)
        .ok()
        .and_then(|index| config.confirm_events.get(index))
    else {
        return;
    };

    if let Some(script_events) = script_events {
        script_events.publish(ScriptEvent::new(topic.clone(), vec![next.to_string()]));
    }
}

fn apply_camera_follow_mode(
    camera_follow: &CameraFollow2dSceneService,
    config: &amigo_behavior::CameraFollowModeControllerBehavior,
) {
    let mut command = camera_follow.follow(&config.camera).unwrap_or_else(|| {
        CameraFollow2dSceneCommand::new(
            "behavior",
            config.camera.clone(),
            config
                .target
                .clone()
                .unwrap_or_else(|| config.camera.clone()),
            amigo_math::Vec2::ZERO,
            config.lerp.unwrap_or(1.0),
        )
    });

    if let Some(target) = config.target.as_ref() {
        command.target = target.clone();
    }
    if let Some(lerp) = config.lerp {
        command.lerp = lerp;
    }
    if let Some(value) = config.lookahead_velocity_scale {
        command.lookahead_velocity_scale = value;
    }
    if let Some(value) = config.lookahead_max_distance {
        command.lookahead_max_distance = value;
    }
    if let Some(value) = config.sway_amount {
        command.sway_amount = value;
    }
    if let Some(value) = config.sway_frequency {
        command.sway_frequency = value;
    }

    camera_follow.queue(command);
}

fn write_menu_selection_state(
    scene_state: &SceneStateService,
    config: &amigo_behavior::MenuNavigationControllerBehavior,
    selected_index: i64,
    item_count: i64,
) {
    let Some(prefix) = config.selected_color_prefix.as_deref() else {
        return;
    };

    for index in 0..item_count {
        let color = if index == selected_index {
            &config.selected_color
        } else {
            &config.unselected_color
        };
        scene_state.set_string(format!("{prefix}.{index}"), color.clone());
    }
}

fn fire_projectile_from_behavior(
    scene: &SceneService,
    motion: &Motion2dSceneService,
    pools: &EntityPoolSceneService,
    physics: Option<&Physics2dSceneService>,
    lifetimes: Option<&LifetimeSceneService>,
    emitter: &str,
    source: &str,
) -> bool {
    let Some(command) = motion.projectile_emitter(emitter) else {
        return false;
    };
    let Some(source_transform) = scene.transform_of(source) else {
        return false;
    };
    let source_velocity = physics
        .and_then(|service| service.body_state(source))
        .map(|state| state.velocity)
        .unwrap_or_else(|| motion.current_velocity(source));
    let Some(projectile_entity) = pools.acquire(scene, &command.emitter.pool) else {
        return false;
    };

    let launch = projectile_launch_2d(source_transform, source_velocity, &command.emitter);
    let _ = scene.set_transform(&projectile_entity, launch.transform);
    let _ = motion.set_velocity(&projectile_entity, launch.velocity);
    if let Some(lifetimes) = lifetimes {
        let _ = lifetimes.reset_lifetime(&projectile_entity);
    }
    if let Some(physics) = physics {
        if let Some(mut body_state) = physics.body_state(&projectile_entity) {
            body_state.velocity = launch.velocity;
            let _ = physics.sync_body_state(&projectile_entity, body_state);
        }
    }

    true
}

fn behavior_condition_matches(
    condition: Option<&amigo_behavior::BehaviorCondition>,
    scene_state: Option<&SceneStateService>,
) -> bool {
    let Some(condition) = condition else {
        return true;
    };
    let Some(scene_state) = scene_state else {
        return false;
    };

    let Some(value) = scene_state_value_as_string(scene_state, &condition.state_key) else {
        return false;
    };

    if let Some(expected) = condition.equals.as_deref() {
        if value != expected {
            return false;
        }
    }

    if let Some(rejected) = condition.not_equals.as_deref() {
        if value == rejected {
            return false;
        }
    }

    if condition.is_true && !scene_state.get_bool(&condition.state_key).unwrap_or(false) {
        return false;
    }

    if condition.is_false && scene_state.get_bool(&condition.state_key).unwrap_or(true) {
        return false;
    }

    if condition.greater_than.is_some()
        || condition.greater_or_equal.is_some()
        || condition.less_than.is_some()
        || condition.less_or_equal.is_some()
    {
        let Some(numeric_value) = scene_state_value_as_f64(scene_state, &condition.state_key)
        else {
            return false;
        };

        if let Some(threshold) = condition.greater_than {
            if numeric_value <= threshold {
                return false;
            }
        }
        if let Some(threshold) = condition.greater_or_equal {
            if numeric_value < threshold {
                return false;
            }
        }
        if let Some(threshold) = condition.less_than {
            if numeric_value >= threshold {
                return false;
            }
        }
        if let Some(threshold) = condition.less_or_equal {
            if numeric_value > threshold {
                return false;
            }
        }
    }

    condition.equals.is_some()
        || condition.not_equals.is_some()
        || condition.is_true
        || condition.is_false
        || condition.greater_than.is_some()
        || condition.greater_or_equal.is_some()
        || condition.less_than.is_some()
        || condition.less_or_equal.is_some()
}

fn scene_state_value_as_string(scene_state: &SceneStateService, key: &str) -> Option<String> {
    if let Some(value) = scene_state.get_string(key) {
        return Some(value);
    }
    if let Some(value) = scene_state.get_bool(key) {
        return Some(value.to_string());
    }
    if let Some(value) = scene_state.get_int(key) {
        return Some(value.to_string());
    }
    if let Some(value) = scene_state.get_float(key) {
        return Some(value.to_string());
    }

    None
}

fn scene_state_value_as_f64(scene_state: &SceneStateService, key: &str) -> Option<f64> {
    if let Some(value) = scene_state.get_float(key) {
        return Some(value);
    }
    if let Some(value) = scene_state.get_int(key) {
        return Some(value as f64);
    }
    if let Some(value) = scene_state.get_string(key) {
        return value.parse::<f64>().ok();
    }

    None
}

fn set_scene_state_from_string(state: &SceneStateService, key: String, value: String) {
    if value.eq_ignore_ascii_case("true") {
        state.set_bool(key, true);
    } else if value.eq_ignore_ascii_case("false") {
        state.set_bool(key, false);
    } else if let Ok(value) = value.parse::<i64>() {
        state.set_int(key, value);
    } else if let Ok(value) = value.parse::<f64>() {
        state.set_float(key, value);
    } else {
        state.set_string(key, value);
    };
}

fn cycle_theme(ui_theme: &UiThemeService) {
    let themes = ui_theme.themes();
    if themes.is_empty() {
        return;
    }

    let active = ui_theme.active_theme_id();
    let active_index = active
        .as_deref()
        .and_then(|active_id| themes.iter().position(|theme| theme.id == active_id))
        .unwrap_or(usize::MAX);
    let next_index = if active_index == usize::MAX {
        0
    } else {
        (active_index + 1) % themes.len()
    };

    if let Some(theme) = themes.get(next_index) {
        ui_theme.set_active_theme(&theme.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_behavior::BehaviorCondition;

    #[test]
    fn behavior_condition_matches_string_scene_state() {
        let scene_state = SceneStateService::default();
        scene_state.set_string("game_mode", "playing");

        assert!(behavior_condition_matches(
            Some(&BehaviorCondition {
                state_key: "game_mode".to_owned(),
                equals: Some("playing".to_owned()),
                not_equals: None,
                greater_than: None,
                greater_or_equal: None,
                less_than: None,
                less_or_equal: None,
                is_true: false,
                is_false: false,
            }),
            Some(&scene_state),
        ));
    }

    #[test]
    fn behavior_condition_rejects_mismatched_scene_state() {
        let scene_state = SceneStateService::default();
        scene_state.set_string("game_mode", "menu");

        assert!(!behavior_condition_matches(
            Some(&BehaviorCondition {
                state_key: "game_mode".to_owned(),
                equals: Some("playing".to_owned()),
                not_equals: None,
                greater_than: None,
                greater_or_equal: None,
                less_than: None,
                less_or_equal: None,
                is_true: false,
                is_false: false,
            }),
            Some(&scene_state),
        ));
    }

    #[test]
    fn behavior_condition_supports_not_equals() {
        let scene_state = SceneStateService::default();
        scene_state.set_string("game_mode", "menu");

        assert!(behavior_condition_matches(
            Some(&BehaviorCondition {
                state_key: "game_mode".to_owned(),
                equals: None,
                not_equals: Some("playing".to_owned()),
                greater_than: None,
                greater_or_equal: None,
                less_than: None,
                less_or_equal: None,
                is_true: false,
                is_false: false,
            }),
            Some(&scene_state),
        ));
    }

    #[test]
    fn behavior_condition_supports_numeric_thresholds() {
        let scene_state = SceneStateService::default();
        scene_state.set_float("charge", 0.75);

        assert!(behavior_condition_matches(
            Some(&BehaviorCondition {
                state_key: "charge".to_owned(),
                equals: None,
                not_equals: None,
                greater_than: Some(0.5),
                greater_or_equal: None,
                less_than: None,
                less_or_equal: Some(1.0),
                is_true: false,
                is_false: false,
            }),
            Some(&scene_state),
        ));

        assert!(!behavior_condition_matches(
            Some(&BehaviorCondition {
                state_key: "charge".to_owned(),
                equals: None,
                not_equals: None,
                greater_than: Some(0.9),
                greater_or_equal: None,
                less_than: None,
                less_or_equal: None,
                is_true: false,
                is_false: false,
            }),
            Some(&scene_state),
        ));
    }

    #[test]
    fn behavior_condition_supports_bool_checks() {
        let scene_state = SceneStateService::default();
        scene_state.set_bool("debug_visible", true);

        assert!(behavior_condition_matches(
            Some(&BehaviorCondition {
                state_key: "debug_visible".to_owned(),
                equals: None,
                not_equals: None,
                greater_than: None,
                greater_or_equal: None,
                less_than: None,
                less_or_equal: None,
                is_true: true,
                is_false: false,
            }),
            Some(&scene_state),
        ));

        assert!(!behavior_condition_matches(
            Some(&BehaviorCondition {
                state_key: "debug_visible".to_owned(),
                equals: None,
                not_equals: None,
                greater_than: None,
                greater_or_equal: None,
                less_than: None,
                less_or_equal: None,
                is_true: false,
                is_false: true,
            }),
            Some(&scene_state),
        ));
    }
}
