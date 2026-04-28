let cube = world.entities.named("playground-3d-cube");

fn on_enter() {
    let scene_id = world.scene.current_id();
    world.dev.event("playground-3d.scene-ready", scene_id);
    world.dev.log("playground-3d scene bootstrapped from scene document for scene=" + scene_id);
}

fn update(dt) {
    let speed = 2.0;

    if world.input.down("ArrowLeft") {
        cube.rotate_3d(0.0, -speed * dt, 0.0);
    }

    if world.input.down("ArrowRight") {
        cube.rotate_3d(0.0, speed * dt, 0.0);
    }

    if world.input.down("ArrowUp") {
        cube.rotate_3d(-speed * dt, 0.0, 0.0);
    }

    if world.input.down("ArrowDown") {
        cube.rotate_3d(speed * dt, 0.0, 0.0);
    }
}
