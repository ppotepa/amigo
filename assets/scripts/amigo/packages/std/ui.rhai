fn theme(world, id) {
    world.ui.set_theme(id)
}

fn text(world, path, value) {
    world.ui.set_text(path, value)
}

fn value(world, path, v) {
    world.ui.set_value(path, v)
}

fn visible(world, path, enabled) {
    if enabled {
        world.ui.show(path);
    } else {
        world.ui.hide(path);
    }
}

fn enabled(world, path, flag) {
    if flag {
        world.ui.enable(path);
    } else {
        world.ui.disable(path);
    }
}
