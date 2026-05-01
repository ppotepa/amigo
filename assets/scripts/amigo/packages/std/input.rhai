fn axis(world, action) {
    world.actions.axis(action)
}

fn down(world, action) {
    world.actions.down(action)
}

fn pressed(world, action) {
    world.actions.pressed(action)
}

fn active_map(world) {
    world.actions.active_map()
}

fn set_active_map(world, map_id) {
    world.actions.set_active_map(map_id)
}
