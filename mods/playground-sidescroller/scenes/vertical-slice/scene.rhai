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

fn on_enter() {
    world.dev.log("playground-sidescroller vertical-slice bootstrapped");
    world.ui.set_text("playground-sidescroller-hud.root.lives", "Lives: " + lives);
    world.ui.set_text("playground-sidescroller-hud.root.time", "Time: 0s");
    world.ui.set_text("playground-sidescroller-hud.root.coins", "Coins: 0 / " + total_coins);
    world.ui.set_text(
        "playground-sidescroller-hud.root.debug",
        "grounded=false vx=0 vy=0 anim=idle",
    );
    world.ui.set_text("playground-sidescroller-hud.root.message", "READY");
    world.audio.start_realtime("proximity-beep");
    world.audio.set_param(
        "proximity-beep",
        "distance",
        world.entities.distance(player.name(), finish.name()),
    );
    player_run_elapsed = 0.0;
    player_run_frame = 1;
    world.sprite2d.set_frame(player.name(), 0);
    for coin_name in coin_names {
        world.sprite2d.set_frame(coin_name, 0);
    }
}

fn update(dt) {
    let move_x = 0.0;

    if world.input.down("ArrowLeft") || world.input.down("A") {
        move_x = move_x - 1.0;
    }

    if world.input.down("ArrowRight") || world.input.down("D") {
        move_x = move_x + 1.0;
    }

    let jump_pressed =
        world.input.pressed("Space") || world.input.pressed("ArrowUp") || world.input.pressed("W");
    let jump_held =
        world.input.down("Space") || world.input.down("ArrowUp") || world.input.down("W");

    let drove = world.platformer.drive(player.name(), move_x, jump_pressed, jump_held, dt);
    let state = world.platformer.state(player.name());
    let grounded = if state.grounded { "true" } else { "false" };
    let proximity = world.entities.distance(player.name(), finish.name());
    if proximity >= 0.0 {
        world.audio.set_param("proximity-beep", "distance", proximity);
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

    world.ui.set_text(
        "playground-sidescroller-hud.root.time",
        "Time: " + world.time.seconds() + "s",
    );
    world.ui.set_text(
        "playground-sidescroller-hud.root.debug",
        "grounded=" + grounded
            + " vx="
            + state.velocity_x_int
            + " vy="
            + state.velocity_y_int
            + " anim="
            + state.animation,
    );
}

fn on_event(topic, payload) {
    if topic == "coin.collected" {
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
        world.ui.set_text("playground-sidescroller-hud.root.message", "LEVEL COMPLETE");
        world.audio.play("level-complete");
        world.audio.stop("proximity-beep");
    }
}
