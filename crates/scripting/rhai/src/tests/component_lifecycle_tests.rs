use super::*;

#[test]
fn script_can_write_trace_entries() {
    let trace = Arc::new(ScriptTraceService::default());
    let runtime = RhaiScriptRuntime::new_with_services_and_ui_theme_and_particle_presets(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(trace.clone()),
    );

    runtime
        .execute(
            "trace-test",
            r#"
                    if !world.trace.begin("drive_actor") { throw("trace begin failed"); }
                    world.trace.value("thrust", 1.0);
                    world.trace.value("turn", -1);
                    world.trace.value("armed", true);
                    if !world.trace.end() { throw("trace end failed"); }
                "#,
        )
        .expect("script execution should succeed");

    let entries = trace.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].label, "drive_actor");
    assert_eq!(
        entries[0].values,
        vec![
            ("thrust".to_owned(), "1".to_owned()),
            ("turn".to_owned(), "-1".to_owned()),
            ("armed".to_owned(), "true".to_owned()),
        ]
    );
}

#[test]
fn exposes_scene_state_and_timers_to_scripts() {
    let state = Arc::new(SceneStateService::default());
    let session = Arc::new(SessionStateService::default());
    let timers = Arc::new(SceneTimerService::default());
    let runtime = RhaiScriptRuntime::new_with_services(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(state.clone()),
        Some(session.clone()),
        Some(timers.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    runtime
            .execute(
                "state-and-timers-test",
                r#"
                    if !world.state.set_int("score", 10) { throw("set_int failed"); }
                    if world.state.add_int("score", 5) != 15 { throw("add_int failed"); }
                    if world.state.get_int("score") != 15 { throw("get_int failed"); }

                    if !world.state.set_float("speed", 1.5) { throw("set_float failed"); }
                    if world.state.add_float("speed", 0.25) != 1.75 { throw("add_float failed"); }
                    if world.state.get_float("speed") != 1.75 { throw("get_float failed"); }

                    if !world.state.set_bool("armed", false) { throw("set_bool failed"); }
                    if !world.state.add_bool("armed", true) { throw("add_bool failed"); }
                    if !world.state.get_bool("armed") { throw("get_bool failed"); }

                    if !world.state.set_string("label", "wave") { throw("set_string failed"); }
                    if world.state.add_string("label", " 1") != "wave 1" { throw("add_string failed"); }
                    if world.state.get_string("label") != "wave 1" { throw("get_string failed"); }

                    if !world.session.set_bool("demo.low_mode", true) { throw("session set_bool failed"); }
                    if !world.session.get_bool("demo.low_mode") { throw("session get_bool failed"); }
                    if !world.session.set_int("demo.highscore.1", 10000) { throw("session set_int failed"); }
                    if world.session.add_int("demo.highscore.1", 250) != 10250 { throw("session add_int failed"); }

                    if !world.timers.start("cooldown", 0.5) { throw("timer start failed"); }
                    if !world.timers.active("cooldown") { throw("timer should be active"); }
                    if world.timers.ready("cooldown") { throw("timer should not be ready"); }

                    fn update(dt) {
                        if world.time.frame() == 1 && !world.timers.ready("cooldown") {
                            throw("timer should be ready after runtime tick");
                        }
                    }
                "#,
            )
            .expect("script execution should succeed");

    runtime
        .call_update("state-and-timers-test", 0.5)
        .expect("update should tick timers before script update");

    assert_eq!(state.get_int("score"), Some(15));
    assert_eq!(session.get_bool("demo.low_mode"), Some(true));
    assert_eq!(session.get_int("demo.highscore.1"), Some(10_250));
    assert!(timers.ready("cooldown"));
}

#[test]
fn script_can_control_particle_emitter() {
    let particles = Arc::new(Particle2dSceneService::default());
    particles.queue_emitter(ParticleEmitter2dCommand {
        entity_id: SceneEntityId::new(44),
        entity_name: "emitter".to_owned(),
        emitter: test_particle_emitter(),
    });
    let runtime = RhaiScriptRuntime::new_with_services(
        None,
        None,
        None,
        None,
        Some(particles.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    runtime
        .execute(
            "particles-test",
            r#"
                    fn update(dt) {
                        world.particles.copy_config("emitter", "emitter");
                        if !world.particles.set_active("emitter", true) {
                            throw("expected particle emitter to exist");
                        }
                        world.particles.set_intensity("emitter", 0.75);
                        world.particles.set_gravity("emitter", 0.0, -120.0);
                        world.particles.set_drag("emitter", 0.5);
                        world.particles.set_wind("emitter", 20.0, 0.0, 0.25);
                        world.particles.set_max_particles("emitter", 12);
                        world.particles.set_lifetime_jitter("emitter", 0.25);
                        world.particles.set_speed_jitter("emitter", 7.0);
                        world.particles.set_local_direction_degrees("emitter", 45.0);
                        world.particles.set_inherit_parent_velocity("emitter", 0.4);
                        world.particles.set_z_index("emitter", 9.0);
                        world.particles.set_spawn_area_rect("emitter", 20.0, 10.0);
                        world.particles.set_spawn_area_line("emitter", 18.0);
                        world.particles.set_spawn_area_ring("emitter", 4.0, 12.0);
                        world.particles.set_shape_line("emitter", 11.0);
                        world.particles.set_shape_mix("emitter", 2.0, 1.0, 1.0);
                        world.particles.set_align("emitter", "emitter");
                        world.particles.set_blend_mode("emitter", "additive");
                        world.particles.set_color_ramp4(
                            "emitter",
                            "linear_rgb",
                            0.0, "FFFFFFFF",
                            0.33, "39D7FFFF",
                            0.66, "246DFFFF",
                            1.0, "00000000"
                        );
                        world.particles.burst("emitter", 3);
                        world.particles.burst_at("emitter", 12.0, -8.0, 2);
                        let yaml = world.particles.export_yaml("emitter");
                        if !yaml.contains("type: ParticleEmitter2D") {
                            throw("expected exported particle yaml");
                        }
                    }
                "#,
        )
        .expect("script execution should succeed");
    runtime
        .call_update("particles-test", 1.0 / 60.0)
        .expect("update should succeed");

    assert!(particles.is_active("emitter"));
    assert_eq!(particles.intensity("emitter"), 0.75);
    assert_eq!(particles.particle_count("emitter"), 0);
    let emitter = particles.emitter("emitter").expect("emitter should exist");
    assert_eq!(emitter.emitter.max_particles, 12);
    assert_eq!(emitter.emitter.forces.len(), 3);
    assert_eq!(
        emitter.emitter.shape_choices[1].shape,
        ParticleShape2d::Line { length: 14.0 }
    );
    assert_eq!(emitter.emitter.shape_choices.len(), 3);
    assert_eq!(
        emitter.emitter.align,
        amigo_2d_particles::ParticleAlignMode2d::Emitter
    );
    assert_eq!(
        emitter.emitter.blend_mode,
        amigo_2d_particles::ParticleBlendMode2d::Additive
    );
    assert_eq!(emitter.emitter.lifetime_jitter, 0.25);
    assert_eq!(emitter.emitter.speed_jitter, 7.0);
    assert_eq!(emitter.emitter.z_index, 9.0);
    assert!((emitter.emitter.local_direction_radians.to_degrees() - 45.0).abs() < 0.001);
    assert_eq!(emitter.emitter.inherit_parent_velocity, 0.4);
    assert!(emitter.emitter.color_ramp.is_some());
}

#[test]
fn script_can_switch_ui_theme() {
    let themes = Arc::new(UiThemeService::default());
    themes.register_theme(UiTheme::from_palette(
        "contrast_dark",
        UiThemePalette {
            background: ColorRgba::new(0.0, 0.0, 0.0, 1.0),
            surface: ColorRgba::new(0.1, 0.1, 0.15, 1.0),
            surface_alt: ColorRgba::new(0.15, 0.15, 0.2, 1.0),
            text: ColorRgba::WHITE,
            text_muted: ColorRgba::new(0.6, 0.7, 0.8, 1.0),
            border: ColorRgba::new(0.2, 0.4, 0.6, 1.0),
            accent: ColorRgba::new(0.0, 0.8, 1.0, 1.0),
            accent_text: ColorRgba::new(0.0, 0.05, 0.08, 1.0),
            danger: ColorRgba::new(1.0, 0.1, 0.2, 1.0),
            warning: ColorRgba::new(1.0, 0.7, 0.0, 1.0),
            success: ColorRgba::new(0.2, 1.0, 0.5, 1.0),
        },
    ));
    let runtime = RhaiScriptRuntime::new_with_services_and_ui_theme(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(themes.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    runtime
        .execute(
            "ui-theme-test",
            r#"
                    if !world.ui.set_theme("contrast_dark") {
                        throw("theme should switch");
                    }
                    if world.ui.theme() != "contrast_dark" {
                        throw("theme should be readable");
                    }
                "#,
        )
        .expect("script execution should succeed");

    assert_eq!(themes.active_theme_id().as_deref(), Some("contrast_dark"));
}

#[test]
fn timers_after_can_be_driven_by_script_tick_and_reset() {
    let timers = Arc::new(SceneTimerService::default());
    let runtime = RhaiScriptRuntime::new_with_services(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(timers.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    runtime
        .execute(
            "timer-after-test",
            r#"
                    if world.timers.after("spawn", 0.25) { throw("after should start pending"); }
                    world.timers.tick(0.25);
                    if !world.timers.after("spawn", 0.25) { throw("after should fire once"); }
                    if world.timers.active("spawn") { throw("after should consume timer"); }
                    if !world.timers.start("reset-me", 1.0) { throw("start reset timer failed"); }
                    world.timers.reset_scene();
                    if world.timers.active("reset-me") { throw("reset should clear scene timers"); }
                "#,
        )
        .expect("script execution should succeed");

    assert!(!timers.active("spawn"));
    assert!(!timers.active("reset-me"));
}
