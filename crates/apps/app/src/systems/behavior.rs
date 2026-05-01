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
    EntityPoolSceneService, LifetimeSceneService, SceneCommand, SceneCommandQueue, SceneKey,
    SceneService,
};
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
            BehaviorKind::SceneTransitionController(config) => {
                if actions.pressed(input.as_ref(), &config.action) {
                    if let Some(scene_commands) = scene_commands.as_ref() {
                        scene_commands.submit(SceneCommand::SelectScene {
                            scene: SceneKey::new(config.scene),
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

    if let Some(value) = scene_state.get_string(&condition.state_key) {
        return value == condition.equals;
    }
    if let Some(value) = scene_state.get_bool(&condition.state_key) {
        return value.to_string() == condition.equals;
    }
    if let Some(value) = scene_state.get_int(&condition.state_key) {
        return value.to_string() == condition.equals;
    }
    if let Some(value) = scene_state.get_float(&condition.state_key) {
        return value.to_string() == condition.equals;
    }

    false
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
                equals: "playing".to_owned(),
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
                equals: "playing".to_owned(),
            }),
            Some(&scene_state),
        ));
    }
}
