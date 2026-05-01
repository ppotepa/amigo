import "pkg:amigo.std/input" as input;

let player = world.entities.named("craterbound-player-1");
let last_debug_text = "";
let last_hp_text = "";
let last_weapon_text = "";
let aim_angle = 0;

fn sync_static_hud() {
    last_hp_text = "HP 100";
    last_weapon_text = "PISTOL INF";
    world.ui.set_text("craterbound-match-hud.root.title", "CRATERBOUND");
    world.ui.set_text("craterbound-match-hud.root.hp", last_hp_text);
    world.ui.set_text("craterbound-match-hud.root.weapon", last_weapon_text);
    world.ui.set_text("craterbound-match-hud.root.mode", "SINGLE PLAYER FIRST PASS");
}

fn on_enter() {
    world.state.reset_scene();
    world.dev.log("craterbound match bootstrapped");
    aim_angle = 0;
    last_debug_text = "";
    sync_static_hud();
}

fn update(dt) {
    let move_x = input::axis(world, "player.move");
    let jump_pressed = input::pressed(world, "player.jump_pressed");
    let jump_held = input::down(world, "player.jump_down");

    if input::pressed(world, "player.aim_up") {
        aim_angle = aim_angle - 5;
        if aim_angle < -70 {
            aim_angle = -70;
        }
    }

    if input::pressed(world, "player.aim_down") {
        aim_angle = aim_angle + 5;
        if aim_angle > 70 {
            aim_angle = 70;
        }
    }

    world.motion.drive(player.name(), move_x, jump_pressed, jump_held, dt);
    let state = world.motion.state(player.name());
    let grounded = if state.grounded { "true" } else { "false" };
    let debug_text =
        "grounded="
            + grounded
            + " vx="
            + state.velocity_x_int
            + " vy="
            + state.velocity_y_int
            + " aim="
            + aim_angle;

    if debug_text != last_debug_text {
        world.ui.set_text("craterbound-match-hud.root.debug", debug_text);
        last_debug_text = debug_text;
    }
}
