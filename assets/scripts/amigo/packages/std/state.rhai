fn int(world, key) {
    world.state.get_int(key)
}

fn set_int(world, key, value) {
    world.state.set_int(key, value)
}

fn float(world, key) {
    world.state.get_float(key)
}

fn set_float(world, key, value) {
    world.state.set_float(key, value)
}

fn bool(world, key) {
    world.state.get_bool(key)
}

fn set_bool(world, key, value) {
    world.state.set_bool(key, value)
}

fn string(world, key) {
    world.state.get_string(key)
}

fn set_string(world, key, value) {
    world.state.set_string(key, value)
}
