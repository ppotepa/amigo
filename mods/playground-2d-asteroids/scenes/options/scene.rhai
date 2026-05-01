fn low_mode(world_api) { world_api.session.get_bool("asteroids.low_mode") }

fn sync_options(world_api) {
    let low = "LOW: OFF";
    if low_mode(world_api) {
        low = "LOW: ON";
    }
    world_api.state.set_string("option_low_text", low);
    world_api.state.set_string("option_back_text", "BACK");
}

fn on_enter() {
    world.state.reset_scene();
    world.state.set_int("option_index", 0);
    world.state.set_string("option_color.0", "#FFFFFFFF");
    world.state.set_string("option_color.1", "#9A9A9AFF");
    world.audio.preload("menu-move");
    world.audio.preload("menu-select");
    sync_options(world);
}

fn on_event(topic, payload) {
    if topic == "asteroids.options.toggle-low" {
        world.session.set_bool("asteroids.low_mode", !low_mode(world));
        sync_options(world);
    }
}

fn update(dt) {}
