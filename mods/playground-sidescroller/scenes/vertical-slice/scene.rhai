import "pkg:amigo.std/input" as input;

let player = world.entities.named("playground-sidescroller-player");
let finish = world.entities.named("playground-sidescroller-finish");
let coin_names = [
    "playground-sidescroller-coin-01",
    "playground-sidescroller-coin-02",
    "playground-sidescroller-coin-03",
    "playground-sidescroller-coin-04",
    "playground-sidescroller-coin-05",
    "playground-sidescroller-coin-06",
    "playground-sidescroller-coin-07",
    "playground-sidescroller-coin-08",
    "playground-sidescroller-coin-09",
    "playground-sidescroller-coin-10",
    "playground-sidescroller-coin-11",
    "playground-sidescroller-coin-12",
    "playground-sidescroller-coin-13",
    "playground-sidescroller-coin-14",
    "playground-sidescroller-coin-15",
    "playground-sidescroller-coin-16",
    "playground-sidescroller-coin-17",
    "playground-sidescroller-coin-18",
    "playground-sidescroller-coin-19",
    "playground-sidescroller-coin-20",
    "playground-sidescroller-coin-21",
    "playground-sidescroller-coin-22",
    "playground-sidescroller-coin-23",
    "playground-sidescroller-coin-24",
    "playground-sidescroller-coin-25",
];
let total_coins = 25;
let lives = 3;
let score = 0;
let player_run_elapsed = 0.0;
let player_run_frame = 1;
let proximity_beep_active = false;
let last_proximity_distance = -1.0;
let proximity_beep_range = 960.0;
let proximity_beep_step = 96.0;
let last_time_seconds = -1;
let debug_elapsed = 0.0;
let last_debug_text = "";
let level_complete = false;
let collected_coins = [];

fn on_enter() {
    world.dev.log("playground-sidescroller vertical-slice bootstrapped");
    world.audio.preload("jump");
    world.audio.preload("coin");
    world.audio.preload("level-complete");
    world.audio.preload("proximity-beep");
    world.audio.play("jump");
    world.ui.set_text("playground-sidescroller-hud.root.lives", "Lives: " + lives);
    world.ui.set_text("playground-sidescroller-hud.root.time", "Time: 0s");
    world.ui.set_text("playground-sidescroller-hud.root.coins", "Coins: 0 / " + total_coins);
    world.ui.set_text(
        "playground-sidescroller-hud.root.debug",
        "grounded=false vx=0 vy=0 anim=idle",
    );
    world.ui.set_text("playground-sidescroller-hud.root.message", "READY");
    let proximity = world.entities.distance(player.name(), finish.name());
    level_complete = false;
    collected_coins = [];
    proximity_beep_active = false;
    last_proximity_distance = -1.0;
    if proximity >= 0.0 && proximity <= proximity_beep_range {
        world.audio.start_realtime("proximity-beep");
        world.audio.set_param("proximity-beep", "distance", proximity);
        proximity_beep_active = true;
        last_proximity_distance = proximity;
    }
    player_run_elapsed = 0.0;
    player_run_frame = 1;
    last_time_seconds = 0;
    debug_elapsed = 0.0;
    last_debug_text = "grounded=false vx=0 vy=0 anim=idle";
    world.sprite2d.set_frame(player.name(), 0);
    for coin_name in coin_names {
        world.sprite2d.set_frame(coin_name, 0);
    }
}

fn update(dt) {
    let move_x = input::axis(world, "player.move");
    let jump_pressed = input::pressed(world, "player.jump_pressed");
    let jump_held = input::down(world, "player.jump_down");

    let drove = world.motion.drive(player.name(), move_x, jump_pressed, jump_held, dt);
    let state = world.motion.state(player.name());
    let grounded = if state.grounded { "true" } else { "false" };
    let proximity = world.entities.distance(player.name(), finish.name());
    if level_complete {
        if proximity_beep_active {
            world.audio.stop("proximity-beep");
            proximity_beep_active = false;
            last_proximity_distance = -1.0;
        }
    } else if proximity >= 0.0 && proximity <= proximity_beep_range {
        if !proximity_beep_active {
            world.audio.start_realtime("proximity-beep");
            proximity_beep_active = true;
            last_proximity_distance = -1.0;
        }

        if last_proximity_distance < 0.0
            || proximity > last_proximity_distance + proximity_beep_step
            || proximity < last_proximity_distance - proximity_beep_step
        {
            world.audio.set_param("proximity-beep", "distance", proximity);
            last_proximity_distance = proximity;
        }
    } else if proximity_beep_active {
        world.audio.stop("proximity-beep");
        proximity_beep_active = false;
        last_proximity_distance = -1.0;
    }

    for coin_name in coin_names {
        world.sprite2d.advance(coin_name, dt);
    }

    if state.animation == "run" {
        player_run_elapsed = player_run_elapsed + dt;
        while player_run_elapsed >= 0.2 {
            player_run_elapsed = player_run_elapsed - 0.2;
            if player_run_frame == 1 {
                player_run_frame = 2;
            } else {
                player_run_frame = 1;
            }
        }
        world.sprite2d.set_frame(player.name(), player_run_frame);
    } else if state.animation == "jump" || state.animation == "fall" {
        player_run_elapsed = 0.0;
        player_run_frame = 1;
        world.sprite2d.set_frame(player.name(), 3);
    } else {
        player_run_elapsed = 0.0;
        player_run_frame = 1;
        world.sprite2d.set_frame(player.name(), 0);
    }

    let time_seconds = world.time.seconds();
    if time_seconds != last_time_seconds {
        world.ui.set_text(
            "playground-sidescroller-hud.root.time",
            "Time: " + time_seconds + "s",
        );
        last_time_seconds = time_seconds;
    }

    debug_elapsed = debug_elapsed + dt;
    if debug_elapsed >= 0.2 {
        debug_elapsed = 0.0;
        let debug_text =
            "grounded=" + grounded
                + " vx="
                + state.velocity_x_int
                + " vy="
                + state.velocity_y_int
                + " anim="
                + state.animation;
        if debug_text != last_debug_text {
            world.ui.set_text("playground-sidescroller-hud.root.debug", debug_text);
            last_debug_text = debug_text;
        }
    }
}

fn on_event(topic, payload) {
    if topic == "coin.collected" {
        if payload.len > 0 {
            let collected_coin = payload[0];
            for collected in collected_coins {
                if collected == collected_coin {
                    return;
                }
            }
            collected_coins.push(collected_coin);
        }

        score = score + 1;
        world.ui.set_text(
            "playground-sidescroller-hud.root.coins",
            "Coins: " + score + " / " + total_coins,
        );
        world.ui.set_text("playground-sidescroller-hud.root.message", "COIN COLLECTED");
        if payload.len > 0 {
            let collected_coin = payload[0];
            world.entities.set_position_2d(collected_coin, -10000.0, -10000.0);
        }
        world.audio.play("coin");
    }

    if topic == "player.jump" {
        world.ui.set_text("playground-sidescroller-hud.root.message", "JUMP");
        world.audio.play("jump");
    }

    if topic == "finish.reached" {
        if level_complete {
            return;
        }

        level_complete = true;
        world.ui.set_text("playground-sidescroller-hud.root.message", "LEVEL COMPLETE");
        world.audio.play("level-complete");
        world.audio.stop("proximity-beep");
        proximity_beep_active = false;
        last_proximity_distance = -1.0;
    }
}
