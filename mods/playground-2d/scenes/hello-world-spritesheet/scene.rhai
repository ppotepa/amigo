let spritesheet = world.entities.named("playground-2d-spritesheet");

fn update(dt) {
    world.sprite2d.advance(spritesheet.name(), dt);

    if world.input.pressed("ArrowDown") {
        world.dev.event("playground-2d.switch.square");
    }
}
