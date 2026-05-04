let elapsed = 0.0;
let last_frame = -1;

fn on_enter() {
    elapsed = 0.0;
    last_frame = -1;
    world.ui.set_text("animated-font-ui.root.pulse", "FRAME 0");
}

fn update(dt) {
    elapsed += dt;
    let frame = (elapsed * 6.0).to_int() % 3;
    if frame != last_frame {
        last_frame = frame;
        world.ui.set_text("animated-font-ui.root.pulse", "FRAME " + frame);
    }
}
