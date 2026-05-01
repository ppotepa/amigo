import "pkg:amigo.std/input" as input;

let cube = world.entities.named("playground-3d-cube");

fn on_enter() {
    let scene_id = world.scene.current_id();
    world.dev.event("playground-3d.scene-ready", scene_id);
    world.dev.log("playground-3d scene bootstrapped from scene document for scene=" + scene_id);
}

fn update(dt) {
    let speed = 2.0;
    let rotate_y = input::axis(world, "cube.rotate_y");
    let rotate_x = input::axis(world, "cube.rotate_x");

    cube.rotate_3d(rotate_x * speed * dt, rotate_y * speed * dt, 0.0);
}
