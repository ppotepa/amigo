fn start(world, id, seconds) {
    world.timers.start(id, seconds)
}

fn active(world, id) {
    world.timers.active(id)
}

fn ready(world, id) {
    world.timers.ready(id)
}

fn after(world, id, seconds) {
    world.timers.after(id, seconds)
}
