let square = world.entities.named("playground-2d-square");

fn update(dt) {
    let speed = 2.0;

    if world.input.down("ArrowLeft") {
        square.rotate_2d(-speed * dt);
    }

    if world.input.down("ArrowRight") {
        square.rotate_2d(speed * dt);
    }

    if world.input.pressed("ArrowUp") {
        world.scene.select("hello-world-spritesheet");
    }

    ()
}
