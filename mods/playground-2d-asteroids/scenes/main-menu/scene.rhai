import "pkg:amigo.std/input" as input;

fn menu_len() { 4 }

fn menu_item(index) {
    if index == 0 { return "START"; }
    if index == 1 { return "OPTIONS"; }
    if index == 2 { return "HIGHSCORES"; }
    return "QUIT";
}

fn menu_index(world_api) { world_api.state.get_int("menu_index") }
fn set_menu_index(world_api, value) { world_api.state.set_int("menu_index", value); }
fn menu_path(id) { "playground-2d-asteroids-main-menu.root." + id }

fn menu_label(world_api, index, label) {
    return label;
}

fn sync_menu_item_color(world_api, index, id) {
    if menu_index(world_api) == index {
        world_api.ui.set_color(menu_path(id), "#FFFFFFFF");
    } else {
        world_api.ui.set_color(menu_path(id), "#9A9A9AFF");
    }
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
    sync_menu_item_color(world_api, 0, "start");
    sync_menu_item_color(world_api, 1, "options");
    sync_menu_item_color(world_api, 2, "highscores");
    sync_menu_item_color(world_api, 3, "quit");
}

fn move_menu(world_api, delta) {
    let next = menu_index(world_api) + delta;
    if next < 0 { next = menu_len() - 1; }
    if next >= menu_len() { next = 0; }
    set_menu_index(world_api, next);
    world_api.audio.play("menu-move");
}

fn activate_menu_item(world_api) {
    let item = menu_item(menu_index(world_api));
    world_api.audio.play("menu-select");

    if item == "START" {
        world_api.scene.select("game");
        return;
    }
    if item == "OPTIONS" {
        world_api.scene.select("options");
        return;
    }
    if item == "HIGHSCORES" {
        world_api.scene.select("highscores");
        return;
    }

    world_api.state.set_string("status_text", "QUIT UNAVAILABLE");
}

fn on_enter() {
    world.state.reset_scene();
    world.state.set_int("menu_index", 0);
    world.state.set_string("status_text", "");
    seed_highscores(world);
    world.audio.preload("menu-move");
    world.audio.preload("menu-select");
    sync_menu(world);
}

fn update(dt) {
    if input::pressed(world, "menu.up") {
        move_menu(world, -1);
    }

    if input::pressed(world, "menu.down") {
        move_menu(world, 1);
    }

    if input::pressed(world, "menu.confirm") {
        activate_menu_item(world);
        return;
    }

    sync_menu(world);
}
