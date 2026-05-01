import "pkg:amigo.std/input" as input;

fn on_enter() {
    world.state.reset_scene();
    world.state.set_string("active_theme", "space_dark");
}

fn update(dt) {}
