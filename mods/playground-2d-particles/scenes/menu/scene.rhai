fn on_enter() {
    world.ui.set_theme("space_dark");
}

fn update(dt) {
    if world.input.pressed("Digit1") || world.input.pressed("Key1") || world.input.pressed("1") {
        world.scene.select("showcase");
        return;
    }

    if world.input.pressed("Digit2") || world.input.pressed("Key2") || world.input.pressed("2") {
        world.scene.select("editor");
        return;
    }

    if world.input.pressed("Escape") {
        world.scene.select("showcase");
        return;
    }
}

fn on_event(topic, payload) {
    if topic == "playground-2d-particles.open-showcase" {
        world.scene.select("showcase");
        return;
    }

    if topic == "playground-2d-particles.open-editor" {
        world.scene.select("editor");
        return;
    }
}
