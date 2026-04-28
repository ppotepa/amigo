let player = world.entities.named("playground-sidescroller-player");
let coin = world.entities.named("playground-sidescroller-coin");
let finish = world.entities.named("playground-sidescroller-finish");
let score = 0;
let player_run_elapsed = 0.0;
let player_run_frame = 1;

fn on_enter() {
    world.dev.log("playground-sidescroller vertical-slice bootstrapped");
    world.ui.set_text("playground-sidescroller-hud.root.score", "Coins: 0");
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
    world.sprite2d.set_frame(coin.name(), 0);
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
    world.sprite2d.advance(coin.name(), dt);
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
        "playground-sidescroller-hud.root.debug",
        "grounded=" + grounded
            + " vx="
            + state.velocity_x
            + " vy="
            + state.velocity_y
            + " anim="
            + state.animation,
    );
}

fn on_event(topic, payload) {
    if topic == "coin.collected" {
        score = score + 1;
        world.ui.set_text("playground-sidescroller-hud.root.score", "Coins: " + score);
        world.ui.set_text("playground-sidescroller-hud.root.message", "COIN COLLECTED");
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
