fn option_len() { 2 }
fn option_index(world_api) { world_api.state.get_int("option_index") }
fn set_option_index(world_api, value) { world_api.state.set_int("option_index", value); }
fn low_mode(world_api) { world_api.session.get_bool("asteroids.low_mode") }

fn label(world_api, index, text) {
    if option_index(world_api) == index {
        return "> " + text + " <";
    }
    return "  " + text;
}

fn sync_options(world_api) {
    let low = "LOW: OFF";
    if low_mode(world_api) {
        low = "LOW: ON";
    }
    world_api.state.set_string("option_low_text", label(world_api, 0, low));
    world_api.state.set_string("option_back_text", label(world_api, 1, "BACK"));
}

fn move_option(world_api, delta) {
    let next = option_index(world_api) + delta;
    if next < 0 { next = option_len() - 1; }
    if next >= option_len() { next = 0; }
    set_option_index(world_api, next);
    world_api.audio.play("menu-move");
}

fn activate_option(world_api) {
    if option_index(world_api) == 0 {
        world_api.session.set_bool("asteroids.low_mode", !low_mode(world_api));
        world_api.audio.play("menu-select");
        sync_options(world_api);
        return;
    }
    world_api.audio.play("menu-select");
    world_api.scene.select("main-menu");
}

fn on_enter() {
    world.state.reset_scene();
    world.state.set_int("option_index", 0);
    world.audio.preload("menu-move");
    world.audio.preload("menu-select");
    sync_options(world);
}

fn update(dt) {
    if world.input.any_pressed(["Escape", "Backspace"]) {
        world.scene.select("main-menu");
        return;
    }
    if world.input.any_pressed(["ArrowUp", "W"]) {
        move_option(world, -1);
    }
    if world.input.any_pressed(["ArrowDown", "S"]) {
        move_option(world, 1);
    }
    if world.input.any_pressed(["Enter", "Space"]) {
        activate_option(world);
        return;
    }
    sync_options(world);
}
