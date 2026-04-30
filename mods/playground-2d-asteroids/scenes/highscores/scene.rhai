fn pad_score(value) {
    if value < 10 { return "00000" + value; }
    if value < 100 { return "0000" + value; }
    if value < 1000 { return "000" + value; }
    if value < 10000 { return "00" + value; }
    if value < 100000 { return "0" + value; }
    return "" + value;
}

fn highscore(world_api, index) {
    let value = world_api.session.get_int("asteroids.highscore." + index);
    if value > 0 { return value; }
    if index == 1 { return 10000; }
    if index == 2 { return 7500; }
    if index == 3 { return 5000; }
    if index == 4 { return 2500; }
    return 1000;
}

fn sync_highscores(world_api) {
    let text = "AAA " + pad_score(highscore(world_api, 1))
        + "\nBBB " + pad_score(highscore(world_api, 2))
        + "\nCCC " + pad_score(highscore(world_api, 3))
        + "\nDDD " + pad_score(highscore(world_api, 4))
        + "\nEEE " + pad_score(highscore(world_api, 5));
    world_api.state.set_string("highscores_text", text);
}

fn on_enter() {
    world.state.reset_scene();
    sync_highscores(world);
}

fn update(dt) {
    if world.input.any_pressed(["Escape", "Backspace", "Space", "Enter"]) {
        world.scene.select("main-menu");
    }
}
