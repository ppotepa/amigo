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
