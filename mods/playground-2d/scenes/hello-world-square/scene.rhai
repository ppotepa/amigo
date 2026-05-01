import "pkg:amigo.std/input" as input;

let square = world.entities.named("playground-2d-square");

fn update(dt) {
    let speed = 2.0;

    if input::down(world, "square.move_left") {
        square.rotate_2d(-speed * dt);
    }

    if input::down(world, "square.move_right") {
        square.rotate_2d(speed * dt);
    }

    ()
}
