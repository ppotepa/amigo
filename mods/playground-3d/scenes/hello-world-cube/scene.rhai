let cube = world.entities.named("playground-3d-cube");

fn on_enter() {
    let scene_id = world.scene.current_id();
    world.dev.event("playground-3d.scene-ready", scene_id);
    world.dev.log("playground-3d scene bootstrapped from scene document for scene=" + scene_id);
}

fn update(dt) {
    let speed = 1.0 / 3.0;
    cube.rotate_3d(speed * dt, speed * 0.67 * dt, speed * 1.31 * dt);
}
