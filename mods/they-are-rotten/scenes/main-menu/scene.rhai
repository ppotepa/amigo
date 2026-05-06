fn seed_menu_state(world_api) {
    world_api.state.set_int("menu_index", 0);

    world_api.state.set_string("menu_color.0", "#39D7FFFF");
    world_api.state.set_string("menu_color.1", "#7D8796FF");
    world_api.state.set_string("menu_color.2", "#7D8796FF");

    world_api.state.set_string("menu_status", "READY");
    world_api.state.set_string("menu_status_color", "#7D8796FF");
}

fn on_enter() {
    world.state.reset_scene();
    seed_menu_state(world);
    world.dev.log("They Are Rotten main menu loaded.");
}

fn update(dt) {}

fn on_event(topic, payload) {
    if topic == "they-are-rotten.menu.start" {
        world.dev.log("Main Menu: START selected.");
        return;
    }

    if topic == "they-are-rotten.menu.options" {
        world.dev.log("Main Menu: OPTIONS selected.");
        return;
    }

    if topic == "they-are-rotten.menu.quit" {
        world.dev.log("Main Menu: QUIT selected.");
        return;
    }
}
