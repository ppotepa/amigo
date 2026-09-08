fn on_event(topic, payload) {
    let p = "world.npr.settings.NprSettings.";
    let actions = ["fit", "focus_selected", "undo", "redo", "capture_before",
        "reset_rotation", "reset_transform", "layout_grid", "camera_front", "camera_side", "camera_top", "reset_style", "new_gesture_variant", "begin_construction_mark", "commit_construction_mark", "close_construction_mark", "cancel_construction_mark", "delete_selected_construction_mark", "select_previous_construction_mark", "select_next_construction_mark", "step"];
    for action in actions {
        if topic == "npr." + action { world.controls.set(p + action, true); return; }
    }
    if topic == "npr.speed_half" { world.controls.set(p + "speed", 0.5); }
    if topic == "npr.speed_normal" { world.controls.set(p + "speed", 1.0); }
    if topic == "npr.speed_double" { world.controls.set(p + "speed", 2.0); }
    let domain = world.controls.get(p + "preset_domain");
    if topic == "npr.save" { world.presets.save(domain, world.controls.get(p + "preset_name"), false); }
    if topic == "npr.overwrite" { world.presets.save(domain, world.controls.get(p + "preset_name"), true); }
    if topic == "npr.load" { world.presets.load(domain, world.controls.get(p + "preset_name")); }
    if topic == "npr.reset" { world.presets.reset("npr-playground"); }
}
