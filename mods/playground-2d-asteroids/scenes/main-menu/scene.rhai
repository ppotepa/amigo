fn menu_label(world_api, index, label) {
    return label;
}

fn seed_highscores(world_api) {
    if world_api.session.get_int("asteroids.highscore.1") <= 0 {
        world_api.session.set_int("asteroids.highscore.1", 10000);
        world_api.session.set_int("asteroids.highscore.2", 7500);
        world_api.session.set_int("asteroids.highscore.3", 5000);
        world_api.session.set_int("asteroids.highscore.4", 2500);
        world_api.session.set_int("asteroids.highscore.5", 1000);
    }
}

fn sync_menu(world_api) {
    world_api.state.set_string("menu_start_text", menu_label(world_api, 0, "START"));
    world_api.state.set_string("menu_options_text", menu_label(world_api, 1, "OPTIONS"));
    world_api.state.set_string("menu_highscores_text", menu_label(world_api, 2, "HIGHSCORES"));
    world_api.state.set_string("menu_quit_text", menu_label(world_api, 3, "QUIT"));
}

fn on_enter() {
    world.state.reset_scene();
    world.state.set_int("menu_index", 0);
    world.state.set_string("status_text", "");
    world.state.set_string("menu_color.0", "#FFFFFFFF");
    world.state.set_string("menu_color.1", "#9A9A9AFF");
    world.state.set_string("menu_color.2", "#9A9A9AFF");
    world.state.set_string("menu_color.3", "#9A9A9AFF");
    seed_highscores(world);
    world.audio.preload("menu-move");
    world.audio.preload("menu-select");
    sync_menu(world);
}

fn update(dt) {}
