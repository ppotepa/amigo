import "pkg:amigo.std/input" as input;

fn ship_name() { "playground-2d-asteroids-ship" }
fn shield_name() { "playground-2d-asteroids-ship-shield" }
fn thrust_name() { "playground-2d-asteroids-main-thruster" }
fn starfield_name() { "playground-2d-asteroids-speed-starfield" }
fn bullet_pool() { "player-bullets" }
fn asteroid_pool() { "asteroids" }

fn game_over_name() { "playground-2d-asteroids-game-over" }
fn hud_name() { "playground-2d-asteroids-hud" }
fn status_name() { "playground-2d-asteroids-center-status" }
fn ui_root(entity_name) { entity_name + ".root" }

fn shield_seconds() { 3.0 }
fn wave_clear_delay() { 1.2 }
fn status_flash_seconds() { 0.95 }
fn ship_spawn_rotation() { 1.5707963 }

fn mode(world_api) { world_api.state.get_string("game_mode") }
fn set_mode(world_api, value) { world_api.state.set_string("game_mode", value); }
fn score(world_api) { world_api.state.get_int("score") }
fn set_score(world_api, value) { world_api.state.set_int("score", value); }
fn lives(world_api) { world_api.state.get_int("lives") }
fn set_lives(world_api, value) { world_api.state.set_int("lives", value); }
fn wave(world_api) { world_api.state.get_int("wave") }
fn set_wave(world_api, value) { world_api.state.set_int("wave", value); }
fn low_mode(world_api) { world_api.session.get_bool("asteroids.low_mode") }
fn rng_seed(world_api) { world_api.state.get_int("rng_seed") }
fn set_rng_seed(world_api, value) { world_api.state.set_int("rng_seed", value); }

fn pad_two(value) {
    if value < 10 { return "0" + value; }
    return "" + value;
}

fn pad_score(value) {
    if value < 10 { return "00000" + value; }
    if value < 100 { return "0000" + value; }
    if value < 1000 { return "000" + value; }
    if value < 10000 { return "00" + value; }
    if value < 100000 { return "0" + value; }
    return "" + value;
}

fn clamp_int(value, min_value, max_value) {
    if value < min_value { return min_value; }
    if value > max_value { return max_value; }
    return value;
}

fn max_float(left, right) {
    if left > right { return left; }
    return right;
}

fn abs_float(value) {
    if value < 0.0 { return -value; }
    return value;
}

fn clamp_float(value, min_value, max_value) {
    if value < min_value { return min_value; }
    if value > max_value { return max_value; }
    return value;
}

fn thrust_hold(world_api) { world_api.state.get_float("ship.thrust_hold"); }
fn set_thrust_hold(world_api, value) { world_api.state.set_float("ship.thrust_hold", value); }

fn rand_next(world_api) {
    let next = (rng_seed(world_api) * 1664525 + 1013904223) % 4294967296;
    if next < 0 { next = -next; }
    set_rng_seed(world_api, next);
    return next;
}

fn rand01(world_api) {
    return rand_next(world_api) / 4294967296.0;
}

fn rand_range(world_api, min_value, max_value) {
    return min_value + (max_value - min_value) * rand01(world_api);
}

fn rand_int(world_api, min_value, max_value) {
    let span = max_value - min_value + 1;
    return min_value + (rand_next(world_api) % span);
}

fn sync_hud(world_api) {
    world_api.state.set_string("score_text", pad_score(score(world_api)));
    world_api.state.set_string("lives_text", "LIVES " + lives(world_api));
    world_api.state.set_string("wave_text", "WAVE " + pad_two(wave(world_api)));
    world_api.state.set_string("final_score_text", "SCORE " + pad_score(score(world_api)));
}

fn set_status(world_api, value) {
    world_api.state.set_bool("status_auto_clear", false);
    world_api.state.set_string("status_text", value);
    if value == "" {
        world_api.entities.hide(status_name());
        world_api.ui.hide(ui_root(status_name()));
    } else {
        world_api.entities.show(status_name());
        world_api.ui.show(ui_root(status_name()));
    }
}

fn set_timed_status(world_api, value, seconds) {
    set_status(world_api, value);
    world_api.state.set_bool("status_auto_clear", true);
    world_api.timers.start("status-clear", seconds);
}

fn update_status(world_api) {
    if world_api.state.get_bool("status_auto_clear") && world_api.timers.ready("status-clear") {
        world_api.state.set_bool("status_auto_clear", false);
        set_status(world_api, "");
    }
}

fn show_gameplay_ui(world_api) {
    world_api.entities.hide(game_over_name());
    world_api.ui.hide(ui_root(game_over_name()));
    world_api.entities.show(hud_name());
    world_api.ui.show(ui_root(hud_name()));
}

fn highscore(world_api, index) {
    return world_api.session.get_int("asteroids.highscore." + index);
}

fn set_highscore(world_api, index, value) {
    world_api.session.set_int("asteroids.highscore." + index, value);
}

fn maybe_insert_highscore(world_api, value) {
    if value > highscore(world_api, 1) {
        set_highscore(world_api, 5, highscore(world_api, 4));
        set_highscore(world_api, 4, highscore(world_api, 3));
        set_highscore(world_api, 3, highscore(world_api, 2));
        set_highscore(world_api, 2, highscore(world_api, 1));
        set_highscore(world_api, 1, value);
        return;
    }
    if value > highscore(world_api, 2) {
        set_highscore(world_api, 5, highscore(world_api, 4));
        set_highscore(world_api, 4, highscore(world_api, 3));
        set_highscore(world_api, 3, highscore(world_api, 2));
        set_highscore(world_api, 2, value);
        return;
    }
    if value > highscore(world_api, 3) {
        set_highscore(world_api, 5, highscore(world_api, 4));
        set_highscore(world_api, 4, highscore(world_api, 3));
        set_highscore(world_api, 3, value);
        return;
    }
    if value > highscore(world_api, 4) {
        set_highscore(world_api, 5, highscore(world_api, 4));
        set_highscore(world_api, 4, value);
        return;
    }
    if value > highscore(world_api, 5) {
        set_highscore(world_api, 5, value);
    }
}

fn reset_player(world_api) {
    world_api.entities.set_position_2d(ship_name(), 0.0, 0.0);
    world_api.entities.set_position_2d(shield_name(), 0.0, 0.0);
    world_api.entities.set_rotation_2d(ship_name(), ship_spawn_rotation());
    world_api.entities.set_rotation_2d(shield_name(), ship_spawn_rotation());
    world_api.motion.reset_freeflight(ship_name(), ship_spawn_rotation());
    world_api.motion.reset_freeflight(shield_name(), ship_spawn_rotation());
    world_api.motion.set_velocity(ship_name(), 0.0, 0.0);
    world_api.motion.set_velocity(shield_name(), 0.0, 0.0);
    world_api.particles.set_active(thrust_name(), false);
    world_api.particles.set_intensity(thrust_name(), 0.0);
    world_api.particles.set_active(starfield_name(), false);
    world_api.particles.set_intensity(starfield_name(), 0.0);
    set_thrust_hold(world_api, 0.0);
    world_api.entities.show(ship_name());
    world_api.entities.enable(ship_name());
    world_api.entities.set_collision_enabled(ship_name(), true);
    world_api.entities.show(shield_name());
    world_api.timers.start("ship-shield", shield_seconds());
}

fn start_new_game(world_api) {
    world_api.timers.reset_scene();
    set_score(world_api, 0);
    set_lives(world_api, 3);
    set_wave(world_api, 1);
    set_rng_seed(world_api, 1337);
    if low_mode(world_api) {
        set_rng_seed(world_api, 7331);
    }

    world_api.pools.release_all(bullet_pool());
    world_api.pools.release_all(asteroid_pool());
    show_gameplay_ui(world_api);
    sync_hud(world_api);
    set_timed_status(world_api, "READY", status_flash_seconds());
    reset_player(world_api);
    set_mode(world_api, "playing");
    start_wave(world_api, 1);
    world_api.audio.play("wave-start");
}

fn wave_asteroid_count(world_api, current_wave) {
    let base = 3 + current_wave;
    if low_mode(world_api) {
        base = base - 1;
    }
    return clamp_int(base, 3, 16);
}

fn wave_speed_min(world_api, current_wave) {
    let value = 55.0 + current_wave * 8.0;
    if low_mode(world_api) {
        return value * 0.72;
    }
    return value;
}

fn wave_speed_max(world_api, current_wave) {
    let value = 110.0 + current_wave * 12.0;
    if low_mode(world_api) {
        return value * 0.72;
    }
    return value;
}

fn wave_radius_min(current_wave) {
    return max_float(14.0, 34.0 - current_wave * 1.2);
}

fn wave_radius_max() { 44.0 }

fn score_for_radius(radius) {
    if radius >= 34.0 { return 20; }
    if radius >= 22.0 { return 50; }
    return 100;
}

fn smooth_step(value) {
    let t = clamp_float(value, 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn ease_out_cubic(value) {
    let t = clamp_float(value, 0.0, 1.0);
    let inv = 1.0 - t;
    return 1.0 - inv * inv * inv;
}

fn ease_in_out_cubic(value) {
    let t = clamp_float(value, 0.0, 1.0);
    if t < 0.5 {
        return 4.0 * t * t * t;
    }
    let f = -2.0 * t + 2.0;
    return 1.0 - f * f * f / 2.0;
}

fn configure_blue_thrust(world_api, thrust_axis) {
    let energy = clamp_float(thrust_axis, 0.0, 1.0);
    let eased = ease_out_cubic(energy);
    world_api.particles.set_velocity_mode(thrust_name(), "source_inertial");
    world_api.particles.set_spawn_rate(thrust_name(), 95.0 + 48.0 * eased);
    world_api.particles.set_lifetime(thrust_name(), 0.18);
    world_api.particles.set_lifetime_jitter(thrust_name(), 0.05);
    world_api.particles.set_speed(thrust_name(), 118.0 + 34.0 * eased);
    world_api.particles.set_speed_jitter(thrust_name(), 18.0);
    world_api.particles.set_spread_degrees(thrust_name(), 4.0);
    world_api.particles.set_initial_size(thrust_name(), 1.15);
    world_api.particles.set_final_size(thrust_name(), 3.2);
    world_api.particles.set_spawn_area_line(thrust_name(), 7.0);
    world_api.particles.set_shape_line(thrust_name(), 5.5);
    world_api.particles.set_shape_mix(thrust_name(), 0.38, 1.5, 0.0);
    world_api.particles.clear_forces(thrust_name());
    world_api.particles.set_curve4(thrust_name(), "size", 0.45, 1.0, 0.72, 0.16);
    world_api.particles.set_curve4(thrust_name(), "speed", 0.42, 0.72, 1.0, 0.82);
    world_api.particles.set_curve4(thrust_name(), "alpha", 0.95, 0.84, 0.28, 0.0);
    world_api.particles.set_color_ramp4(
        thrust_name(),
        "linear_rgb",
        0.0, "#F8FFFFFF",
        0.22, "#7CE8FFFF",
        0.72, "#246DFFFF",
        1.0, "#00184000"
    );
}

fn configure_green_thrust(world_api, thrust_axis, hold_seconds) {
    let energy = clamp_float(thrust_axis, 0.0, 1.0);
    let burn = ease_in_out_cubic((hold_seconds - 0.28) / 1.05);
    let energy_eased = ease_out_cubic(energy);
    let chaos_phase = (hold_seconds * 18.0).to_int() % 3;
    let chaos = 0.0;
    if chaos_phase == 0 { chaos = 1.0; }
    if chaos_phase == 1 { chaos = 0.42; }
    world_api.particles.set_velocity_mode(thrust_name(), "source_inertial");
    world_api.particles.set_spawn_rate(thrust_name(), 185.0 + 74.0 * energy_eased + 78.0 * burn + 46.0 * chaos);
    world_api.particles.set_lifetime(thrust_name(), 0.34 + 0.24 * burn + 0.055 * chaos);
    world_api.particles.set_lifetime_jitter(thrust_name(), 0.09 + 0.06 * chaos);
    world_api.particles.set_speed(thrust_name(), 190.0 + 58.0 * burn + 36.0 * chaos);
    world_api.particles.set_speed_jitter(thrust_name(), 34.0 + 22.0 * chaos);
    world_api.particles.set_spread_degrees(thrust_name(), 3.2 + 1.2 * chaos - 1.0 * burn);
    world_api.particles.set_initial_size(thrust_name(), 0.75);
    world_api.particles.set_final_size(thrust_name(), 10.0 + 6.0 * burn + 3.0 * chaos);
    world_api.particles.set_spawn_area_line(thrust_name(), 8.0 - 2.2 * burn + 3.0 * chaos);
    world_api.particles.set_shape_line(thrust_name(), 12.0 + 11.0 * burn + 4.0 * chaos);
    world_api.particles.set_shape_mix(thrust_name(), 0.55 + 1.1 * burn + 0.7 * chaos, 2.0 + 0.9 * chaos, 0.0);
    world_api.particles.clear_forces(thrust_name());
    world_api.particles.set_curve4(thrust_name(), "size", 0.04, 0.92, 0.22 + 0.78 * chaos, 0.08);
    world_api.particles.set_curve4(thrust_name(), "speed", 0.95, 0.28 + 0.72 * chaos, 1.0, 0.18);
    world_api.particles.set_curve4(thrust_name(), "alpha", 1.0, 0.46 + 0.44 * chaos, 0.72 - 0.46 * chaos, 0.0);
    world_api.particles.set_color_ramp4(
        thrust_name(),
        "linear_rgb",
        0.0, "#F8FFFFFF",
        0.24, "#6DFFB5FF",
        0.68, "#23FF5ACC",
        1.0, "#00301500"
    );
}

fn configure_ion_pulse_thrust(world_api, thrust_axis, hold_seconds) {
    let energy = clamp_float(thrust_axis, 0.0, 1.0);
    let climax = ease_out_cubic((hold_seconds - 1.28) / 0.72);
    let pulse_phase = (hold_seconds * 12.0).to_int() % 4;
    let pulse = 0.0;
    if pulse_phase == 0 { pulse = 1.0; }
    if pulse_phase == 1 { pulse = 0.35; }
    let pulse_power = pulse * (0.45 + 0.55 * climax);
    let burst_phase = (hold_seconds * 6.0).to_int() % 3;
    if burst_phase == 0 {
        world_api.particles.burst(thrust_name(), 10);
    }
    world_api.particles.set_velocity_mode(thrust_name(), "source_inertial");
    world_api.particles.set_spawn_rate(thrust_name(), 260.0 + 110.0 * energy + 250.0 * pulse_power);
    world_api.particles.set_lifetime(thrust_name(), 0.18 + 0.12 * pulse_power);
    world_api.particles.set_lifetime_jitter(thrust_name(), 0.10 + 0.08 * pulse_power);
    world_api.particles.set_speed(thrust_name(), 360.0 + 160.0 * pulse_power);
    world_api.particles.set_speed_jitter(thrust_name(), 58.0 + 120.0 * pulse_power);
    world_api.particles.set_spread_degrees(thrust_name(), 1.5 + 1.2 * pulse_power);
    world_api.particles.set_initial_size(thrust_name(), 0.32 + 0.45 * pulse_power);
    world_api.particles.set_final_size(thrust_name(), 3.6 + 2.7 * pulse_power);
    world_api.particles.set_spawn_area_line(thrust_name(), 2.2 + 1.6 * pulse_power);
    world_api.particles.set_shape_line(thrust_name(), 46.0 + 42.0 * pulse_power);
    world_api.particles.set_shape_mix(thrust_name(), 0.25 + 0.55 * pulse_power, 5.0 + 3.0 * pulse_power, 0.0);
    world_api.particles.clear_forces(thrust_name());
    world_api.particles.set_curve4(thrust_name(), "size", 0.08, 0.72, 1.0, 0.0);
    world_api.particles.set_curve4(thrust_name(), "speed", 0.62, 1.0, 0.72 + 0.22 * pulse_power, 0.12);
    world_api.particles.set_curve4(thrust_name(), "alpha", 1.0, 0.95, 0.48 + 0.36 * pulse_power, 0.0);
    world_api.particles.set_color_ramp4(
        thrust_name(),
        "linear_rgb",
        0.0, "#FFFFFFFF",
        0.26, "#FFF26AFF",
        0.62, "#FFB000CC",
        1.0, "#3A220000"
    );
}

fn update_thruster_effect(world_api, thrust_axis, dt) {
    if thrust_axis <= 0.01 {
        set_thrust_hold(world_api, 0.0);
        world_api.particles.set_active(thrust_name(), false);
        world_api.particles.set_intensity(thrust_name(), 0.0);
        return;
    }

    let hold_seconds = clamp_float(thrust_hold(world_api) + dt, 0.0, 2.0);
    set_thrust_hold(world_api, hold_seconds);
    world_api.particles.set_active(thrust_name(), true);
    world_api.particles.set_intensity(thrust_name(), clamp_float(thrust_axis, 0.0, 1.0));

    if hold_seconds < 0.28 {
        configure_blue_thrust(world_api, thrust_axis);
        return;
    }
    if hold_seconds < 1.35 {
        configure_green_thrust(world_api, thrust_axis, hold_seconds);
        return;
    }
    configure_ion_pulse_thrust(world_api, thrust_axis, hold_seconds);
}

fn update_speed_starfield(world_api, thrust_axis) {
    let intensity = clamp_float((thrust_axis - 0.82) / 0.18, 0.0, 1.0);
    let ramp_raw = clamp_float((thrust_hold(world_api) - 0.72) / 1.35, 0.0, 1.0);
    let ramp = ease_in_out_cubic(ramp_raw);
    let streak = smooth_step(ramp * ramp);
    let motion = world_api.motion.state(ship_name());
    let speed_sq = motion.velocity_x * motion.velocity_x + motion.velocity_y * motion.velocity_y;

    if intensity <= 0.01 || ramp <= 0.01 || speed_sq < 115000.0 {
        world_api.particles.set_active(starfield_name(), false);
        world_api.particles.set_intensity(starfield_name(), 0.0);
        return;
    }

    let chaos_phase = (thrust_hold(world_api) * 21.0).to_int() % 4;
    let chaos = 0.0;
    if chaos_phase == 0 { chaos = 1.0 * ramp; }
    if chaos_phase == 1 { chaos = 0.45 * ramp; }

    world_api.particles.set_active(starfield_name(), true);
    world_api.particles.set_intensity(starfield_name(), intensity * ramp);
    world_api.particles.set_local_direction_degrees(starfield_name(), motion.angle_radians * 57.29578 + 180.0);
    world_api.particles.set_spawn_rate(starfield_name(), 32.0 + 112.0 * intensity * ramp + 42.0 * chaos);
    world_api.particles.set_lifetime(starfield_name(), 0.26 + 0.42 * ramp + 0.035 * chaos);
    world_api.particles.set_lifetime_jitter(starfield_name(), 0.06 + 0.045 * chaos);
    world_api.particles.set_speed(starfield_name(), 320.0 + 380.0 * intensity * ramp + 72.0 * chaos);
    world_api.particles.set_speed_jitter(starfield_name(), 44.0 + 92.0 * ramp + 32.0 * chaos);
    world_api.particles.set_spread_degrees(starfield_name(), 1.2 + 1.4 * chaos);
    world_api.particles.set_initial_size(starfield_name(), 0.16 + 0.18 * ramp);
    world_api.particles.set_final_size(starfield_name(), 0.55 + 1.35 * streak + 0.35 * chaos);
    world_api.particles.set_shape_line(starfield_name(), 3.0 + 20.0 * streak + 4.0 * chaos);
}

fn split_radius(radius) {
    if radius >= 34.0 { return radius * 0.58; }
    if radius >= 22.0 { return radius * 0.62; }
    return 0.0;
}

fn asteroid_size_for_radius(radius) {
    if radius >= 34.0 { return "large"; }
    if radius >= 22.0 { return "medium"; }
    return "small";
}

fn fragment_speed(world_api, parent_speed) {
    let min_speed = wave_speed_min(world_api, wave(world_api)) * 1.15;
    let speed = parent_speed * 1.18;
    return max_float(min_speed, speed);
}

fn random_spawn_position_away_from_center(world_api) {
    let origin = world_api.entities.position_2d(ship_name());
    let x = rand_range(world_api, -860.0, 860.0);
    let y = rand_range(world_api, -460.0, 460.0);

    if abs_float(x) < 220.0 && abs_float(y) < 180.0 {
        if abs_float(x) < abs_float(y) {
            if y >= 0.0 { y = 260.0; } else { y = -260.0; }
        } else {
            if x >= 0.0 { x = 300.0; } else { x = -300.0; }
        }
    }

    return #{ x: origin.x + x, y: origin.y + y };
}

fn direction_x(index) {
    if index == 0 { return 1.0; }
    if index == 1 { return 0.7071; }
    if index == 2 { return 0.0; }
    if index == 3 { return -0.7071; }
    if index == 4 { return -1.0; }
    if index == 5 { return -0.7071; }
    if index == 6 { return 0.0; }
    return 0.7071;
}

fn direction_y(index) {
    if index == 0 { return 0.0; }
    if index == 1 { return 0.7071; }
    if index == 2 { return 1.0; }
    if index == 3 { return 0.7071; }
    if index == 4 { return 0.0; }
    if index == 5 { return -0.7071; }
    if index == 6 { return -1.0; }
    return -0.7071;
}

fn configure_asteroid(world_api, asteroid, x, y, radius, vertices, speed, direction, seed) {
    world_api.entities.set_position_2d(asteroid, x, y);
    world_api.motion.set_velocity(
        asteroid,
        direction_x(direction) * speed,
        direction_y(direction) * speed,
    );
    world_api.vector.set_radial_jitter_polygon(asteroid, vertices, radius, 0.24, seed);
    world_api.physics.set_circle_radius(asteroid, radius);
    world_api.entities.set_property_string(asteroid, "asteroid_size", asteroid_size_for_radius(radius));
    world_api.entities.set_property_int(asteroid, "score_value", score_for_radius(radius));
    world_api.entities.set_property_float(asteroid, "radius", radius);
    world_api.entities.set_property_float(asteroid, "speed", speed);
}

fn start_wave(world_api, current_wave) {
    world_api.pools.release_all(asteroid_pool());
    let count = wave_asteroid_count(world_api, current_wave);
    let i = 0;

    while i < count {
        let asteroid = world_api.pools.acquire(asteroid_pool());
        if asteroid == "" { return; }

        let radius = rand_range(world_api, wave_radius_min(current_wave), wave_radius_max());
        let vertices = rand_int(world_api, 8, 14);
        let speed = rand_range(world_api, wave_speed_min(world_api, current_wave), wave_speed_max(world_api, current_wave));
        let direction = rand_int(world_api, 0, 7);
        let pos = random_spawn_position_away_from_center(world_api);
        let seed = rng_seed(world_api) + i;

        configure_asteroid(
            world_api,
            asteroid,
            pos.x,
            pos.y,
            radius,
            vertices,
            speed,
            direction,
            seed,
        );

        i = i + 1;
    }

    set_wave(world_api, current_wave);
    sync_hud(world_api);
    set_timed_status(world_api, "WAVE " + pad_two(current_wave), status_flash_seconds());
}

fn enter_wave_clear(world_api) {
    set_mode(world_api, "wave_clear");
    world_api.particles.set_active(thrust_name(), false);
    world_api.particles.set_intensity(thrust_name(), 0.0);
    world_api.particles.set_active(starfield_name(), false);
    world_api.particles.set_intensity(starfield_name(), 0.0);
    set_score(world_api, score(world_api) + wave(world_api) * 100);
    sync_hud(world_api);
    set_status(world_api, "WAVE CLEAR");
    world_api.timers.start("wave-clear", wave_clear_delay());
    world_api.audio.play("wave-start");
}

fn update_wave_clear(world_api) {
    if world_api.timers.ready("wave-clear") {
        let next_wave = wave(world_api) + 1;
        set_mode(world_api, "playing");
        start_wave(world_api, next_wave);
        world_api.audio.play("wave-start");
    }
}

fn enter_game_over(world_api) {
    set_mode(world_api, "game_over");
    world_api.pools.release_all(bullet_pool());
    world_api.pools.release_all(asteroid_pool());
    world_api.entities.hide(ship_name());
    world_api.entities.disable(ship_name());
    world_api.entities.set_collision_enabled(ship_name(), false);
    world_api.particles.set_active(thrust_name(), false);
    world_api.particles.set_intensity(thrust_name(), 0.0);
    world_api.particles.set_active(starfield_name(), false);
    world_api.particles.set_intensity(starfield_name(), 0.0);
    world_api.entities.hide(shield_name());
    maybe_insert_highscore(world_api, score(world_api));
    sync_hud(world_api);
    world_api.entities.hide(hud_name());
    world_api.ui.hide(ui_root(hud_name()));
    world_api.entities.show(game_over_name());
    world_api.ui.show(ui_root(game_over_name()));
    set_status(world_api, "");
    world_api.audio.play("game-over");
}

fn update_game_over(world_api) {}

fn update_shield(world_api) {
    if world_api.timers.active("ship-shield") {
        world_api.entities.show(shield_name());
    } else {
        world_api.entities.hide(shield_name());
    }
}

fn update_player_input(world_api, dt) {
    let thrust_axis = input::axis(world_api, "ship.thrust");
    update_thruster_effect(world_api, thrust_axis, dt);
    update_speed_starfield(world_api, thrust_axis);
}

fn update_playing(world_api, dt) {
    if input::pressed(world_api, "game.restart") {
        start_new_game(world_api);
        return;
    }

    update_player_input(world_api, dt);
    update_shield(world_api);
    update_status(world_api);
}

fn spawn_split_fragment(world_api, x, y, radius, speed, direction, seed) {
    let fragment = world_api.pools.acquire(asteroid_pool());
    if fragment == "" { return false; }

    let offset = radius * 0.55;
    configure_asteroid(
        world_api,
        fragment,
        x + direction_x(direction) * offset,
        y + direction_y(direction) * offset,
        radius,
        rand_int(world_api, 8, 12),
        speed,
        direction,
        seed,
    );
    return true;
}

fn split_asteroid(world_api, asteroid, radius, x, y) {
    let child_radius = split_radius(radius);
    if child_radius <= 0.0 { return 0; }

    let parent_speed = world_api.entities.property_float(asteroid, "speed");
    let speed = fragment_speed(world_api, parent_speed);
    let direction = rand_int(world_api, 0, 7);
    let opposite = (direction + 4) % 8;
    let seed = rng_seed(world_api);
    let spawned = 0;

    if spawn_split_fragment(world_api, x, y, child_radius, speed, direction, seed + 17) {
        spawned = spawned + 1;
    }
    if spawn_split_fragment(world_api, x, y, child_radius, speed, opposite, seed + 31) {
        spawned = spawned + 1;
    }

    return spawned;
}

fn handle_bullet_hit(world_api, bullet, asteroid) {
    let pos = world_api.entities.position_2d(asteroid);
    let radius = world_api.entities.property_float(asteroid, "radius");
    let score_value = world_api.entities.property_int(asteroid, "score_value");

    world_api.pools.release(bullet_pool(), bullet);
    set_score(world_api, score(world_api) + score_value);
    world_api.pools.release(asteroid_pool(), asteroid);
    split_asteroid(world_api, asteroid, radius, pos.x, pos.y);
    world_api.audio.play("asteroid-hit");
    sync_hud(world_api);

    if world_api.pools.active_count(asteroid_pool()) <= 0 {
        enter_wave_clear(world_api);
    }
}

fn handle_ship_hit(world_api) {
    if mode(world_api) != "playing" { return; }
    if world_api.timers.active("ship-shield") { return; }

    set_lives(world_api, lives(world_api) - 1);
    sync_hud(world_api);
    world_api.pools.release_all(bullet_pool());

    if lives(world_api) <= 0 {
        enter_game_over(world_api);
        return;
    }

    set_timed_status(world_api, "READY", status_flash_seconds());
    reset_player(world_api);
    world_api.audio.play("respawn-shield");
}

fn on_enter() {
    world.state.reset_scene();
    world.timers.reset_scene();

    let clips = [
        "shot",
        "asteroid-hit",
        "game-over",
        "wave-start",
        "respawn-shield",
    ];
    let clip_index = 0;
    while clip_index < clips.len() {
        world.audio.preload(clips[clip_index]);
        clip_index = clip_index + 1;
    }

    start_new_game(world);
}

fn on_event(topic, payload) {
    if topic == "asteroids.bullet_hit_asteroid" && payload.len() >= 2 {
        if mode(world) == "playing" {
            handle_bullet_hit(world, payload[0], payload[1]);
        }
        return;
    }

    if topic == "asteroids.ship_hit_asteroid" {
        handle_ship_hit(world);
        return;
    }
}

fn update(dt) {
    let current_mode = mode(world);

    if current_mode == "playing" {
        update_playing(world, dt);
        return;
    }

    if current_mode == "wave_clear" {
        update_wave_clear(world);
        return;
    }

    if current_mode == "game_over" {
        update_game_over(world);
        return;
    }

}
