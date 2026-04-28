let demo_square = world.entities.named("playground-2d-demo-square");
let demo_spritesheet = world.entities.named("playground-2d-demo-spritesheet");

fn on_enter() {
    let available_scenes = world.scene.available();
    let runtime_window = world.runtime.window_backend();
    let runtime_render = world.runtime.render_backend();
    let square_asset = world.assets.get("playground-2d/textures/square");
    let spritesheet_asset = world.assets.get("playground-2d/textures/hello-world-spritesheet");

    world.dev.log("entered basic-scripting-demo for mod=" + world.mod.current_id());
    world.dev.log("runtime=" + runtime_window + "/" + runtime_render);
    world.dev.log("scene-count=" + available_scenes.len);
    world.dev.log("square-asset=" + square_asset.state());
    world.dev.log("spritesheet-asset=" + spritesheet_asset.state());
    world.dev.event("playground-2d.demo.entered", world.scene.current_id());
}

fn update(dt) {
    let speed = 2.0;

    if world.input.down("ArrowLeft") {
        demo_square.rotate_2d(-speed * dt);
    }

    if world.input.down("ArrowRight") {
        demo_square.rotate_2d(speed * dt);
    }

    if world.input.pressed("ArrowUp") {
        let demo_sheet_name = demo_spritesheet.name();
        world.sprite2d.advance(demo_sheet_name, 1.0 / 8.0);
        let frame = world.sprite2d.frame(demo_sheet_name);
        world.dev.log("basic-scripting-demo frame=" + frame);
    }

    if world.input.pressed("ArrowDown") {
        world.sprite2d.set_frame(demo_spritesheet.name(), 0);
        world.dev.log("basic-scripting-demo frame reset");
    }

    if world.input.pressed("W") {
        world.scene.select("hello-world-square");
    }

    if world.input.pressed("S") {
        world.dev.event("playground-2d.demo.goto-spritesheet");
    }

    if world.input.pressed("Space") {
        world.scene.reload();
    }

    if world.input.pressed("Enter") {
        let delta = world.time.delta();
        let elapsed = world.time.elapsed();
        let frame = world.time.frame();

        world.dev.log("time-delta=" + delta);
        world.dev.log("time-elapsed=" + elapsed);
        world.dev.log("time-frame=" + frame);
        world.dev.command("help");
    }

    if world.input.pressed("D") {
        world.dev.refresh_diagnostics(world.mod.current_id());
    }
}
