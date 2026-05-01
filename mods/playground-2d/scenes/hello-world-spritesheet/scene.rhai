import "pkg:amigo.std/input" as input;

let spritesheet = world.entities.named("playground-2d-spritesheet");

fn update(dt) {
    world.sprite2d.advance(spritesheet.name(), dt);

    if input::pressed(world, "spritesheet.return_square") {
        world.dev.event("playground-2d.switch.square");
    }
}
