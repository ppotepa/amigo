let ship = world.entities.named("playground-2d-asteroids-ship");
let thrust = world.entities.named("playground-2d-asteroids-thrust");
let ship_shield = world.entities.named("playground-2d-asteroids-ship-shield");

let ship_name = "playground-2d-asteroids-ship";
let thrust_name = "playground-2d-asteroids-thrust";
let shield_name = "playground-2d-asteroids-ship-shield";
let bullet_pool = "player-bullets";

let player_motion_names = [
    "playground-2d-asteroids-ship",
    "playground-2d-asteroids-thrust",
    "playground-2d-asteroids-ship-shield",
];

let asteroid_names = [
    "playground-2d-asteroids-asteroid-big",
    "playground-2d-asteroids-asteroid-small",
    "playground-2d-asteroids-asteroid-fragment-left",
    "playground-2d-asteroids-asteroid-fragment-right",
    "playground-2d-asteroids-asteroid-wave-02-a",
    "playground-2d-asteroids-asteroid-wave-02-b",
    "playground-2d-asteroids-asteroid-wave-02-c",
];

let asteroid_alive = [true, true, false, false, false, false, false];

let ship_start_x = -540.0;
let ship_start_y = 0.0;

let fire_interval = 0.16;
let thrust_audio_interval = 0.2;
let ship_respawn_grace = 1.25;
let wave_transition_delay = 0.9;
let asteroid_shape_jitter = 0.24;

let score = 0;
let lives = 3;
let wave_index = 1;
let level_clear = false;
let game_over = false;
let wave_transition_pending = false;

fn set_status(world_api, value) {
    world_api.state.set_string("status_text", value);
    let hud = #{};
    hud["playground-2d-asteroids-hud.root.status"] = value;
    world_api.ui.set_many(hud);
}

fn sync_hud(world_api, current_score, current_lives, current_wave) {
    let score_text = "Score: " + current_score;
    let lives_text = "Lives: " + current_lives;
    let wave_text = "Wave: " + current_wave;
    world_api.state.set_string("score_text", score_text);
    world_api.state.set_string("lives_text", lives_text);
    world_api.state.set_string("wave_text", wave_text);
    let hud = #{};
    hud["playground-2d-asteroids-hud.root.score"] = score_text;
    hud["playground-2d-asteroids-hud.root.lives"] = lives_text;
    hud["playground-2d-asteroids-hud.root.wave"] = wave_text;
    world_api.ui.set_many(hud);
}

fn sync_debug(world_api, thrust_axis, turn_axis) {
    let thrust_int = floor(thrust_axis * 100.0 + 0.5) / 100.0;
    let turn_int = floor(turn_axis * 100.0 + 0.5) / 100.0;
    let debug_text = "THRUST: " + thrust_int + " | TURN: " + turn_int;
    world_api.state.set_string("debug_text", debug_text);
    let hud = #{};
    hud["playground-2d-asteroids-hud.root.debug"] = debug_text;
    world_api.ui.set_many(hud);
}

fn set_entity_active(world_api, entity_name, active) {
    if active {
        world_api.entities.show(entity_name);
        world_api.entities.enable(entity_name);
        world_api.entities.set_collision_enabled(entity_name, true);
    } else {
        world_api.motion.set_velocity(entity_name, 0.0, 0.0);
        world_api.entities.hide(entity_name);
        world_api.entities.disable(entity_name);
        world_api.entities.set_collision_enabled(entity_name, false);
    }
}

fn stop_motion(world_api, names) {
    let index = 0;
    while index < names.len() {
        world_api.motion.set_velocity(names[index], 0.0, 0.0);
        index = index + 1;
    }
}

fn reset_player(world_api, names, start_x, start_y, flame_name, guard_name, shield_visible) {
    let index = 0;
    while index < names.len() {
        world_api.entities.set_position_2d(names[index], start_x, start_y);
        index = index + 1;
    }
    stop_motion(world_api, names);
    world_api.entities.hide(flame_name);
    world_api.entities.set_position_2d(guard_name, start_x, start_y);
    if shield_visible {
        world_api.entities.show(guard_name);
    } else {
        world_api.entities.hide(guard_name);
    }
}

fn drive_player_motion(world_api, names, thrust_axis, turn_axis) {
    let intent = #{};
    intent["thrust"] = thrust_axis;
    intent["turn"] = turn_axis;
    intent["strafe"] = 0.0;

    let index = 0;
    while index < names.len() {
        world_api.motion.drive_freeflight(names[index], intent);
        index = index + 1;
    }
}

fn set_asteroid_shape(world_api, entity_name, jitter, seed) {
    world_api.vector.set_radial_jitter_polygon(
        entity_name,
        world_api.entities.property_int(entity_name, "vertices"),
        world_api.entities.property_float(entity_name, "radius"),
        jitter,
        seed,
    );
}

fn asteroid_index(entity_name, names) {
    let index = 0;
    while index < names.len() {
        if names[index] == entity_name {
            return index;
        }
        index = index + 1;
    }
    -1
}

fn wave_one_clear(alive) {
    !alive[0] && !alive[1] && !alive[2] && !alive[3]
}

fn wave_two_clear(alive) {
    !alive[4] && !alive[5] && !alive[6]
}

fn clear_active_bullets(world_api, pool) {
    let active = world_api.pools.active_members(pool);
    let index = 0;
    while index < active.len() {
        world_api.projectiles.release(pool, active[index]);
        index = index + 1;
    }
}

fn activate_asteroid(world_api, names, index, x, y, vx, vy, jitter, seed) {
    world_api.entities.set_position_2d(names[index], x, y);
    world_api.motion.set_velocity(names[index], vx, vy);
    set_entity_active(world_api, names[index], true);
    set_asteroid_shape(world_api, names[index], jitter, seed);
}

fn deactivate_asteroid(world_api, names, index) {
    set_entity_active(world_api, names[index], false);
}

fn reset_asteroids(world_api, names, jitter) {
    activate_asteroid(world_api, names, 0, 380.0, 126.0, -120.0, -86.0, jitter, 11);
    activate_asteroid(world_api, names, 1, 586.0, -136.0, -150.0, 118.0, jitter, 13);

    deactivate_asteroid(world_api, names, 2);
    world_api.entities.set_position_2d(names[2], 0.0, 0.0);
    set_asteroid_shape(world_api, names[2], jitter, 14);

    deactivate_asteroid(world_api, names, 3);
    world_api.entities.set_position_2d(names[3], 0.0, 0.0);
    set_asteroid_shape(world_api, names[3], jitter, 16);

    deactivate_asteroid(world_api, names, 4);
    world_api.entities.set_position_2d(names[4], 512.0, 286.0);
    set_asteroid_shape(world_api, names[4], jitter, 47);

    deactivate_asteroid(world_api, names, 5);
    world_api.entities.set_position_2d(names[5], 252.0, -312.0);
    set_asteroid_shape(world_api, names[5], jitter, 49);

    deactivate_asteroid(world_api, names, 6);
    world_api.entities.set_position_2d(names[6], 642.0, 44.0);
    set_asteroid_shape(world_api, names[6], jitter, 50);
}

fn start_wave_two(world_api, names, jitter) {
    activate_asteroid(world_api, names, 4, 512.0, 286.0, -168.0, -128.0, jitter, 47);
    activate_asteroid(world_api, names, 5, 252.0, -312.0, 152.0, 148.0, jitter, 49);
    activate_asteroid(world_api, names, 6, 642.0, 44.0, -186.0, 88.0, jitter, 50);
}

fn split_large_asteroid(world_api, names, jitter, current_score) {
    activate_asteroid(world_api, names, 2, 356.0, 136.0, -170.0, 126.0, jitter, 83 + current_score);
    activate_asteroid(world_api, names, 3, 404.0, 118.0, 176.0, -128.0, jitter, 84 + current_score);
}

fn on_enter() {
    world.state.reset_scene();
    world.timers.reset_scene();

    let clips = [
        "shot",
        "thrust",
        "asteroid-hit",
        "asteroid-split",
        "ship-hit",
        "game-over",
        "wave-start",
        "sector-clear",
        "respawn-shield",
    ];
    let clip_index = 0;
    while clip_index < clips.len() {
        world.audio.preload(clips[clip_index]);
        clip_index = clip_index + 1;
    }

    score = 0;
    lives = 3;
    wave_index = 1;
    level_clear = false;
    game_over = false;
    wave_transition_pending = false;

    clear_active_bullets(world, bullet_pool);
    reset_player(world, player_motion_names, ship_start_x, ship_start_y, thrust_name, shield_name, true);
    world.timers.start("ship-invulnerable", ship_respawn_grace);
    world.audio.play("respawn-shield");

    asteroid_alive = [true, true, false, false, false, false, false];
    reset_asteroids(world, asteroid_names, asteroid_shape_jitter);

    sync_hud(world, score, lives, wave_index);
    sync_debug(world, 0.0, 0.0);
    set_status(world, "READY // SPACE TO FIRE");
}

fn on_event(topic, payload) {
    if topic == "asteroids.bullet_hit_asteroid" && payload.len() >= 2 {
        world.projectiles.release(bullet_pool, payload[0]);

        let hit_name = payload[1];
        let hit_index = asteroid_index(hit_name, asteroid_names);
        if hit_index >= 0 && asteroid_alive[hit_index] {
            asteroid_alive[hit_index] = false;
            deactivate_asteroid(world, asteroid_names, hit_index);
            score = score + world.entities.property_int(hit_name, "score_value");

            if hit_index == 0 {
                asteroid_alive[2] = true;
                asteroid_alive[3] = true;
                split_large_asteroid(world, asteroid_names, asteroid_shape_jitter, score);
                set_status(world, "BIG ASTEROID SPLIT");
                world.audio.play("asteroid-split");
            } else if hit_index >= 4 {
                set_status(world, "WAVE 2 TARGET DOWN");
                world.audio.play("asteroid-hit");
            } else if hit_index == 1 {
                set_status(world, "SMALL ASTEROID DOWN");
                world.audio.play("asteroid-hit");
            } else {
                set_status(world, "FRAGMENT DOWN");
                world.audio.play("asteroid-hit");
            }

            sync_hud(world, score, lives, wave_index);

            if !game_over && wave_index == 1 && wave_one_clear(asteroid_alive) {
                clear_active_bullets(world, bullet_pool);
                reset_player(world, player_motion_names, ship_start_x, ship_start_y, thrust_name, shield_name, false);
                wave_transition_pending = true;
                world.timers.start("wave-transition", wave_transition_delay);
                set_status(world, "WAVE 1 CLEAR // STANDBY");
            }

            if !game_over && wave_index == 2 && wave_two_clear(asteroid_alive) {
                level_clear = true;
                clear_active_bullets(world, bullet_pool);
                stop_motion(world, player_motion_names);
                thrust.hide();
                ship_shield.hide();
                set_status(world, "SECTOR CLEAR // PRESS R");
                world.audio.play("sector-clear");
            }
        }
    }

    if topic == "asteroids.ship_hit_asteroid"
        && !game_over
        && !level_clear
        && !wave_transition_pending
        && !world.timers.active("ship-invulnerable")
    {
        lives = lives - 1;
        sync_hud(world, score, lives, wave_index);
        clear_active_bullets(world, bullet_pool);
        thrust.hide();

        if lives <= 0 {
            game_over = true;
            stop_motion(world, player_motion_names);
            ship_shield.hide();
            set_status(world, "GAME OVER // PRESS R");
            world.audio.play("game-over");
        } else {
            reset_player(world, player_motion_names, ship_start_x, ship_start_y, thrust_name, shield_name, true);
            world.timers.start("ship-invulnerable", ship_respawn_grace);
            set_status(world, "SHIP HIT");
            world.audio.play("respawn-shield");
            world.audio.play("ship-hit");
        }
    }
}

fn update(dt) {
    if world.input.any_pressed(["R"]) {
        world.scene.reload();
        return;
    }

    if game_over || level_clear {
        if world.input.any_pressed(["Space"]) {
            world.scene.reload();
        }
        return;
    }

    if wave_transition_pending {
        if world.timers.ready("wave-transition") {
            wave_transition_pending = false;
            wave_index = 2;

            reset_player(world, player_motion_names, ship_start_x, ship_start_y, thrust_name, shield_name, true);
            world.timers.start("ship-invulnerable", ship_respawn_grace);
            world.audio.play("respawn-shield");

            asteroid_alive[4] = true;
            asteroid_alive[5] = true;
            asteroid_alive[6] = true;
            start_wave_two(world, asteroid_names, asteroid_shape_jitter);

            sync_hud(world, score, lives, wave_index);
            set_status(world, "WAVE 2 // ENGAGE");
            world.audio.play("wave-start");
        }
        return;
    }

    let thrust_axis = world.input.axis(["ArrowUp", "W"], ["ArrowDown", "S"]);
    let turn_axis = world.input.axis(["ArrowLeft", "A"], ["ArrowRight", "D"]);
    drive_player_motion(world, player_motion_names, thrust_axis, turn_axis);

    if thrust_axis > 0.0 {
        thrust.show();

        if !world.timers.active("thrust-audio") {
            world.audio.play("thrust");
            world.timers.start("thrust-audio", thrust_audio_interval);
        }
    } else {
        thrust.hide();
    }

    if !world.timers.active("fire") && world.input.any_pressed(["Space"]) {
        if world.projectiles.fire_from(ship_name, ship_name) {
            world.timers.start("fire", fire_interval);
            set_status(world, "SHOT FIRED");
            world.audio.play("shot");
        }
    }

    if world.timers.active("ship-invulnerable") {
        ship_shield.show();
    } else {
        ship_shield.hide();
    }

    sync_debug(world, thrust_axis, turn_axis);
}
