import "pkg:amigo.std/input" as input;

fn option_len() { 2 }
fn option_index(world_api) { world_api.state.get_int("option_index") }
fn set_option_index(world_api, value) { world_api.state.set_int("option_index", value); }
fn low_mode(world_api) { world_api.session.get_bool("asteroids.low_mode") }
fn option_path(id) { "playground-2d-asteroids-options.root." + id }

fn label(world_api, index, text) {
    return text;
}

fn sync_option_color(world_api, index, id) {
    if option_index(world_api) == index {
        world_api.ui.set_color(option_path(id), "#FFFFFFFF");
    } else {
        world_api.ui.set_color(option_path(id), "#9A9A9AFF");
    }
}

fn sync_options(world_api) {
    let low = "LOW: OFF";
    if low_mode(world_api) {
        low = "LOW: ON";
    }
    world_api.state.set_string("option_low_text", label(world_api, 0, low));
    world_api.state.set_string("option_back_text", label(world_api, 1, "BACK"));
    sync_option_color(world_api, 0, "low");
    sync_option_color(world_api, 1, "back");
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
    if input::pressed(world, "menu.back") {
        world.scene.select("main-menu");
        return;
    }
    if input::pressed(world, "menu.up") {
        move_option(world, -1);
    }
    if input::pressed(world, "menu.down") {
        move_option(world, 1);
    }
    if input::pressed(world, "menu.confirm") {
        activate_option(world);
        return;
    }
    sync_options(world);
}
