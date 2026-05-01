fn active(world, name, enabled) {
    world.particles.set_active(name, enabled)
}

fn intensity(world, name, value) {
    world.particles.set_intensity(name, value)
}

fn drive_emitter(world, name, value) {
    if value > 0.01 {
        world.particles.start(name);
        world.particles.set_intensity(name, value);
    } else {
        world.particles.stop(name);
    }
}
