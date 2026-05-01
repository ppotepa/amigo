pub(crate) fn tick_behaviors(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
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
            }
            BehaviorKind::ParticleIntensityController(config) => {
                if let Some(particles) = particles.as_ref() {
                    let intensity = actions.axis(input.as_ref(), &config.action).abs();
                    particles.set_active(&config.emitter, intensity > 0.01);
                    particles.set_intensity(&config.emitter, intensity);
                }
            }
            BehaviorKind::ParticleProfileController(config) => {
                if let Some(particles) = particles.as_ref() {
                    tick_particle_profile_controller(
                        behaviors.as_ref(),
                        actions.as_ref(),
                        input.as_ref(),
                        particles.as_ref(),
                        &command.entity_name,
                        &config,
                        delta_seconds,
                    );
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

