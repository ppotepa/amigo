let box = world.entities.named("playground-npr-box-source");
let fox = world.entities.named("playground-npr-fox-source");
let probe = world.entities.named("playground-npr-comic-probe");

fn on_enter() {
    let scene_id = world.scene.current_id();
    world.dev.event("playground-npr.scene-ready", scene_id);
    world.dev.log("playground-npr comic lines workbench bootstrapped for scene=" + scene_id);
}

fn update(dt) {
    box.rotate_3d(0.0, 0.24 * dt, 0.0);
    fox.rotate_3d(0.0, -0.18 * dt, 0.0);
    probe.rotate_3d(0.12 * dt, 0.31 * dt, 0.0);
}
