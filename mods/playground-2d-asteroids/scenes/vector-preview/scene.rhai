fn ship_name() { "playground-2d-asteroids-ship" }
fn shield_name() { "playground-2d-asteroids-ship-shield" }
fn thrust_name() { "playground-2d-asteroids-thrust" }
fn bullet_pool() { "player-bullets" }
fn asteroid_pool() { "asteroids" }

fn main_menu_name() { "playground-2d-asteroids-main-menu" }
fn highscores_name() { "playground-2d-asteroids-highscores" }
fn game_over_name() { "playground-2d-asteroids-game-over" }
fn hud_name() { "playground-2d-asteroids-hud" }
fn status_name() { "playground-2d-asteroids-center-status" }

fn fire_interval() { 0.16 }
fn shield_seconds() { 3.0 }
fn wave_clear_delay() { 1.2 }
fn ship_spawn_rotation() { 1.5707963 }

fn menu_len() { 4 }

fn menu_item(index) {
    if index == 0 { return "START"; }
    if index == 1 { return "HIGHSCORES"; }
    if index == 2 { return "LOW"; }
    return "QUIT";
}

fn mode(world_api) { world_api.state.get_string("game_mode") }
fn set_mode(world_api, value) { world_api.state.set_string("game_mode", value); }
fn score(world_api) { world_api.state.get_int("score") }
fn set_score(world_api, value) { world_api.state.set_int("score", value); }
fn lives(world_api) { world_api.state.get_int("lives") }
fn set_lives(world_api, value) { world_api.state.set_int("lives", value); }
fn wave(world_api) { world_api.state.get_int("wave") }
fn set_wave(world_api, value) { world_api.state.set_int("wave", value); }
fn menu_index(world_api) { world_api.state.get_int("menu_index") }
fn set_menu_index(world_api, value) { world_api.state.set_int("menu_index", value); }
fn low_mode(world_api) { world_api.state.get_bool("low_mode") }
fn set_low_mode(world_api, value) { world_api.state.set_bool("low_mode", value); }
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

fn menu_label(world_api, index, label) {
    if menu_index(world_api) == index {
        return "> " + label + " <";
    }
    return "  " + label;
}

fn sync_menu(world_api) {
    world_api.state.set_string("menu_start_text", menu_label(world_api, 0, "START"));
    world_api.state.set_string("menu_highscores_text", menu_label(world_api, 1, "HIGHSCORES"));

    let low_label = "LOW: OFF";
    if low_mode(world_api) {
        low_label = "LOW: ON";
    }
    world_api.state.set_string("menu_low_text", menu_label(world_api, 2, low_label));
    world_api.state.set_string("menu_quit_text", menu_label(world_api, 3, "QUIT"));
}

fn sync_hud(world_api) {
    world_api.state.set_string("score_text", pad_score(score(world_api)));
    world_api.state.set_string("lives_text", "LIVES " + lives(world_api));
    world_api.state.set_string("wave_text", "WAVE " + pad_two(wave(world_api)));
    world_api.state.set_string("final_score_text", "SCORE " + pad_score(score(world_api)));
}

fn set_status(world_api, value) {
    world_api.state.set_string("status_text", value);
    if value == "" {
        world_api.entities.hide(status_name());
    } else {
        world_api.entities.show(status_name());
    }
}

fn hide_gameplay(world_api) {
    world_api.entities.hide(ship_name());
    world_api.entities.disable(ship_name());
    world_api.entities.set_collision_enabled(ship_name(), false);
    world_api.entities.hide(shield_name());
    world_api.entities.hide(thrust_name());
    world_api.pools.release_all(bullet_pool());
    world_api.pools.release_all(asteroid_pool());
}

fn show_screen(world_api, screen) {
    world_api.entities.hide(main_menu_name());
    world_api.entities.hide(highscores_name());
    world_api.entities.hide(game_over_name());
    world_api.entities.hide(hud_name());

    if screen == "menu" { world_api.entities.show(main_menu_name()); }
    if screen == "highscores" { world_api.entities.show(highscores_name()); }
    if screen == "game_over" { world_api.entities.show(game_over_name()); }
    if screen == "hud" { world_api.entities.show(hud_name()); }
}

fn init_highscores(world_api) {
    world_api.state.set_int("highscore_1", 10000);
    world_api.state.set_int("highscore_2", 7500);
    world_api.state.set_int("highscore_3", 5000);
    world_api.state.set_int("highscore_4", 2500);
    world_api.state.set_int("highscore_5", 1000);
}

fn highscore(world_api, index) {
    return world_api.state.get_int("highscore_" + index);
}

fn set_highscore(world_api, index, value) {
    world_api.state.set_int("highscore_" + index, value);
}

fn sync_highscores(world_api) {
    let text = "AAA " + pad_score(highscore(world_api, 1))
        + "\nBBB " + pad_score(highscore(world_api, 2))
        + "\nCCC " + pad_score(highscore(world_api, 3))
        + "\nDDD " + pad_score(highscore(world_api, 4))
        + "\nEEE " + pad_score(highscore(world_api, 5));
    world_api.state.set_string("highscores_text", text);
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

fn enter_menu(world_api) {
    set_mode(world_api, "menu");
    hide_gameplay(world_api);
    show_screen(world_api, "menu");
    set_status(world_api, "");
    sync_menu(world_api);
    sync_highscores(world_api);
}

fn enter_highscores(world_api) {
    set_mode(world_api, "highscores");
    hide_gameplay(world_api);
    show_screen(world_api, "highscores");
    set_status(world_api, "");
    sync_highscores(world_api);
}

fn enter_quit_pending(world_api) {
    set_mode(world_api, "quit_pending");
    show_screen(world_api, "menu");
    set_status(world_api, "QUIT UNAVAILABLE");
}

fn reset_player(world_api) {
    world_api.entities.set_position_2d(ship_name(), 0.0, 0.0);
    world_api.entities.set_position_2d(shield_name(), 0.0, 0.0);
    world_api.motion.reset_freeflight(ship_name(), ship_spawn_rotation());
    world_api.motion.reset_freeflight(shield_name(), ship_spawn_rotation());
    world_api.motion.set_velocity(ship_name(), 0.0, 0.0);
    world_api.motion.set_velocity(shield_name(), 0.0, 0.0);
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
    show_screen(world_api, "hud");
    sync_hud(world_api);
    set_status(world_api, "READY");
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

fn asteroid_size_for_radius(radius) {
    if radius >= 34.0 { return "large"; }
    if radius >= 22.0 { return "medium"; }
    return "small";
}

fn random_spawn_position_away_from_center(world_api) {
    let x = rand_range(world_api, -860.0, 860.0);
    let y = rand_range(world_api, -460.0, 460.0);

    if abs_float(x) < 220.0 && abs_float(y) < 180.0 {
        if abs_float(x) < abs_float(y) {
            if y >= 0.0 { y = 260.0; } else { y = -260.0; }
        } else {
            if x >= 0.0 { x = 300.0; } else { x = -300.0; }
        }
    }

    return #{ x: x, y: y };
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

        world_api.entities.set_position_2d(asteroid, pos.x, pos.y);
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

        i = i + 1;
    }

    set_wave(world_api, current_wave);
    sync_hud(world_api);
    set_status(world_api, "WAVE " + pad_two(current_wave));
}

fn enter_wave_clear(world_api) {
    set_mode(world_api, "wave_clear");
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
    world_api.entities.hide(shield_name());
    maybe_insert_highscore(world_api, score(world_api));
    sync_highscores(world_api);
    sync_hud(world_api);
    show_screen(world_api, "game_over");
    set_status(world_api, "");
    world_api.audio.play("game-over");
}

fn update_menu(world_api) {
    if world_api.input.any_pressed(["ArrowUp", "W"]) {
        let next = menu_index(world_api) - 1;
        if next < 0 { next = menu_len() - 1; }
        set_menu_index(world_api, next);
        world_api.audio.play("menu-move");
    }

    if world_api.input.any_pressed(["ArrowDown", "S"]) {
        let next = menu_index(world_api) + 1;
        if next >= menu_len() { next = 0; }
        set_menu_index(world_api, next);
        world_api.audio.play("menu-move");
    }

    if world_api.input.any_pressed(["Enter", "Space"]) {
        let item = menu_item(menu_index(world_api));
        if item == "START" {
            world_api.audio.play("menu-select");
            start_new_game(world_api);
            return;
        }
        if item == "HIGHSCORES" {
            world_api.audio.play("menu-select");
            enter_highscores(world_api);
            return;
        }
        if item == "LOW" {
            set_low_mode(world_api, !low_mode(world_api));
            world_api.audio.play("menu-select");
        }
        if item == "QUIT" {
            world_api.audio.play("menu-select");
            enter_quit_pending(world_api);
            return;
        }
    }

    sync_menu(world_api);
}

fn update_highscores(world_api) {
    if world_api.input.any_pressed(["Escape", "Backspace", "Space", "Enter"]) {
        enter_menu(world_api);
    }
}

fn update_game_over(world_api) {
    if world_api.input.any_pressed(["Space", "Enter", "Escape"]) {
        enter_menu(world_api);
    }
}

fn update_quit_pending(world_api) {
    if world_api.input.any_pressed(["Escape", "Backspace", "Space", "Enter"]) {
        enter_menu(world_api);
    }
}

fn update_shield(world_api) {
    if world_api.timers.active("ship-shield") {
        world_api.entities.show(shield_name());
    } else {
        world_api.entities.hide(shield_name());
    }
}

fn update_player_input(world_api) {
    let thrust_axis = world_api.input.axis(["ArrowUp", "W"], ["ArrowDown", "S"]);
    let turn_axis = world_api.input.axis(["ArrowLeft", "A"], ["ArrowRight", "D"]);
    let intent = #{};
    intent["thrust"] = thrust_axis;
    intent["turn"] = turn_axis;
    intent["strafe"] = 0.0;
    world_api.motion.drive_freeflight(ship_name(), intent);
    world_api.motion.drive_freeflight(shield_name(), intent);

    if !world_api.timers.active("fire") && world_api.input.any_pressed(["Space"]) {
        if world_api.projectiles.fire_from(ship_name(), ship_name()) {
            world_api.timers.start("fire", fire_interval());
            world_api.audio.play("shot");
        }
    }
}

fn update_playing(world_api) {
    if world_api.input.any_pressed(["R"]) {
        start_new_game(world_api);
        return;
    }
    if world_api.input.any_pressed(["Escape"]) {
        enter_menu(world_api);
        return;
    }

    update_player_input(world_api);
    update_shield(world_api);
}

fn handle_bullet_hit(world_api, bullet, asteroid) {
    world_api.pools.release(bullet_pool(), bullet);
    set_score(world_api, score(world_api) + world_api.entities.property_int(asteroid, "score_value"));
    world_api.pools.release(asteroid_pool(), asteroid);
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

    set_status(world_api, "READY");
    reset_player(world_api);
    world_api.audio.play("respawn-shield");
}

fn on_enter() {
    world.state.reset_scene();
    world.timers.reset_scene();
    set_mode(world, "menu");
    set_menu_index(world, 0);
    set_score(world, 0);
    set_lives(world, 3);
    set_wave(world, 1);
    set_low_mode(world, false);
    set_rng_seed(world, 1337);
    init_highscores(world);

    let clips = [
        "shot",
        "asteroid-hit",
        "game-over",
        "wave-start",
        "respawn-shield",
        "menu-move",
        "menu-select",
    ];
    let clip_index = 0;
    while clip_index < clips.len() {
        world.audio.preload(clips[clip_index]);
        clip_index = clip_index + 1;
    }

    enter_menu(world);
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

    if current_mode == "menu" {
        update_menu(world);
        return;
    }

    if current_mode == "highscores" {
        update_highscores(world);
        return;
    }

    if current_mode == "playing" {
        update_playing(world);
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

    if current_mode == "quit_pending" {
        update_quit_pending(world);
        return;
    }
}
