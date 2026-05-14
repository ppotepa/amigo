fn reset(world_api) {
    world_api.state.set_string("rain_glass_preset", "scene-default");
    world_api.dev.log("RainGlass presets: press 1-9 or 0 for ten temporary scene presets.");
    world_api.dev.log("RainGlass debug: F1 final, F2 trail_map, F3 blurred_scene, C normals, V mask, B mist, R refraction, T raindrop_map.");
    world_api.dev.log("Color quantize: [ fewer colors, ] more colors.");
}

fn update(world_api) {
    if world_api.input.any_pressed("BracketLeft,[") {
        adjust_palette(world_api, -1);
        return;
    }
    if world_api.input.any_pressed("BracketRight,]") {
        adjust_palette(world_api, 1);
        return;
    }
    if world_api.input.any_pressed("F1") {
        debug_view(world_api, "Final", "final");
        return;
    }
    if world_api.input.any_pressed("F2") {
        debug_view(world_api, "Trail Map", "trail_map");
        return;
    }
    if world_api.input.any_pressed("F3") {
        debug_view(world_api, "Blurred Scene", "blurred_scene");
        return;
    }
    if world_api.input.any_pressed("C,KeyC") {
        debug_view(world_api, "Drop Normals", "drop_normals");
        return;
    }
    if world_api.input.any_pressed("V,KeyV") {
        debug_view(world_api, "Drop Mask", "drop_mask");
        return;
    }
    if world_api.input.any_pressed("B,KeyB") {
        debug_view(world_api, "Mist", "mist");
        return;
    }
    if world_api.input.any_pressed("R,KeyR") {
        debug_view(world_api, "Refraction", "refraction");
        return;
    }
    if world_api.input.any_pressed("T,KeyT") {
        debug_view(world_api, "Raindrop Map", "raindrop_map");
        return;
    }
    if world_api.input.any_pressed("Digit1,Key1,1") {
        reference(world_api);
        return;
    }
    if world_api.input.any_pressed("Digit2,Key2,2") {
        cinematic(world_api);
        return;
    }
    if world_api.input.any_pressed("Digit3,Key3,3") {
        storm_wall(world_api);
        return;
    }
    if world_api.input.any_pressed("Digit4,Key4,4") {
        long_streaks(world_api);
        return;
    }
    if world_api.input.any_pressed("Digit5,Key5,5") {
        fogged_glass(world_api);
        return;
    }
    if world_api.input.any_pressed("Digit6,Key6,6") {
        macro_lens(world_api);
        return;
    }
    if world_api.input.any_pressed("Digit7,Key7,7") {
        micro_beads(world_api);
        return;
    }
    if world_api.input.any_pressed("Digit8,Key8,8") {
        cold_neon(world_api);
        return;
    }
    if world_api.input.any_pressed("Digit9,Key9,9") {
        dry_glass(world_api);
        return;
    }
    if world_api.input.any_pressed("Digit0,Key0,0") {
        debug_refraction(world_api);
        return;
    }
}

fn rg(world_api, args) {
    world_api.postfx.set_rain_glass(args);
}

fn select(world_api, name) {
    world_api.state.set_string("rain_glass_preset", name);
    world_api.dev.log("RainGlass preset: " + name);
}

fn debug_view(world_api, name, value) {
    world_api.postfx.set_rain_glass("debug=" + value);
    world_api.dev.log("RainGlass debug: " + name);
}

fn adjust_palette(world_api, delta) {
    let value = world_api.postfx.adjust_color_quantize_palette_size(delta);
    world_api.dev.log("Color quantize palette: " + value.to_string() + " colors");
}

fn reference(world_api) {
    select(world_api, "Reference");
    rg(world_api, "preset=html_current_controls debug=final seed=121713");
}

fn cinematic(world_api) {
    select(world_api, "Cinematic");
    rg(world_api, "preset=html_cinematic_button debug=final seed=121713");
    rg(world_api, "body=0.58 distortion=56 normal=7.2 focus_blur=1.05");
    rg(world_api, "trail_refract=0.62 trail_opacity=0.78 collider=1.65");
}

fn storm_wall(world_api) {
    select(world_api, "Storm Wall");
    rg(world_api, "preset=html_storm_button debug=final seed=92131");
    rg(world_api, "spawn_rate=28 spawn_limit=1400 radius_min=8 radius_max=82");
    rg(world_api, "gravity=3100 slip_rate=0.42 velocity_stretch=0.58 evaporate=7 collider=1.8");
    rg(world_api, "trails=true trail_density=0.52 trail_spread=1.05 trail_taper=0.54 trail_evaporate=22 trail_shrink_rate=0.92 trail_dist_min=3 trail_dist_max=8");
    rg(world_api, "micro_enabled=true micro=1100 micro_min=4 micro_max=22");
    rg(world_api, "mist=true mist_opacity=0.42 mist_time=10 mist_strength=0.024 mist_blur_step=5 mist_accumulation=0.28");
    rg(world_api, "blur=15 blur_steps=4 refract_base=0.66 refract_scale=2.0 distortion=68 body=0.46 scene_light=2.4 rim=1.75");
}

fn long_streaks(world_api) {
    select(world_api, "Long Streaks");
    rg(world_api, "preset=lens_streaks debug=final seed=33017");
    rg(world_api, "spawn_rate=8 spawn_limit=620 radius_min=34 radius_max=128");
    rg(world_api, "gravity=2400 slip_rate=0.30 velocity_stretch=0.36 collider=1.85 evaporate=10");
    rg(world_api, "trails=true trail_density=0.42 trail_size_min=0.34 trail_size_max=0.72 trail_spread=1.30 streak=0.95 streak_len=2.20 trail_taper=0.74 trail_evaporate=18 trail_shrink_rate=0.94 trail_dist_min=3 trail_dist_max=9");
    rg(world_api, "micro_enabled=true micro=240 micro_min=7 micro_max=24");
    rg(world_api, "mist=true mist_opacity=0.18 mist_time=18 mist_strength=0.009 mist_blur_step=3");
    rg(world_api, "blur=9 blur_steps=2 refract_base=0.45 refract_scale=1.35 distortion=58 body=0.42 trail_refract=0.55 trail_opacity=0.74");
}

fn fogged_glass(world_api) {
    select(world_api, "Fogged Glass");
    rg(world_api, "preset=cinematic debug=final seed=88112");
    rg(world_api, "spawn_rate=4 spawn_limit=260 radius_min=16 radius_max=62");
    rg(world_api, "trails=true trail_density=0.28 trail_spread=0.72 streak=0.42 streak_len=0.85 trail_evaporate=28 trail_shrink_rate=0.91");
    rg(world_api, "micro_enabled=true micro=180 micro_min=5 micro_max=16");
    rg(world_api, "mist=true mist_opacity=0.62 mist_time=24 mist_strength=0.034 mist_blur=22 mist_blur_step=6 mist_accumulation=0.38");
    rg(world_api, "blur=20 blur_steps=5 refract_base=0.28 refract_scale=0.70 distortion=34 body=0.28 scene_light=1.15 rim=0.92");
}

fn macro_lens(world_api) {
    select(world_api, "Macro Lens");
    rg(world_api, "preset=optics_debug debug=final seed=44901");
    rg(world_api, "spawn_rate=1 spawn_limit=18 radius_min=96 radius_max=180");
    rg(world_api, "gravity=900 slip_rate=0.08 initial_spread=0.75 velocity_stretch=0.12 evaporate=4 collider=1.25");
    rg(world_api, "trails=false micro_enabled=false mist=false");
    rg(world_api, "blur=18 blur_steps=4 refract_base=0.95 refract_scale=3.8 distortion=92 normal=10 focus_blur=1.45 body=0.52 scene_light=3.2 rim=2.25 chroma=0.28");
}

fn micro_beads(world_api) {
    select(world_api, "Micro Beads");
    rg(world_api, "preset=subtle debug=final seed=77219");
    rg(world_api, "spawn_rate=7 spawn_limit=900 radius_min=8 radius_max=38");
    rg(world_api, "gravity=1700 slip_rate=0.18 velocity_stretch=0.26 evaporate=18 collider=1.2");
    rg(world_api, "trails=true trail_density=0.24 trail_spread=0.48 streak=0.28 streak_len=0.55 trail_evaporate=32 trail_dist_min=8 trail_dist_max=18");
    rg(world_api, "micro_enabled=true micro=1300 micro_min=3 micro_max=13");
    rg(world_api, "mist=true mist_opacity=0.16 mist_time=20 mist_strength=0.008 mist_blur_step=2");
    rg(world_api, "blur=6 blur_steps=2 refract_base=0.22 refract_scale=0.88 distortion=36 body=0.42 scene_light=1.45 rim=1.18");
}

fn cold_neon(world_api) {
    select(world_api, "Cold Neon");
    rg(world_api, "preset=cinematic debug=final seed=51499");
    rg(world_api, "spawn_rate=10 spawn_limit=760 radius_min=20 radius_max=96");
    rg(world_api, "gravity=2300 slip_rate=0.28 velocity_stretch=0.40 collider=1.55");
    rg(world_api, "trails=true trail_density=0.36 trail_spread=0.88 streak=0.70 streak_len=1.35 trail_taper=0.66 trail_evaporate=20 trail_dist_min=4 trail_dist_max=11");
    rg(world_api, "micro_enabled=true micro=620 micro_min=8 micro_max=27");
    rg(world_api, "mist=true mist_opacity=0.22 mist_time=16 mist_strength=0.012 mist_blur_step=4");
    rg(world_api, "blur=12 blur_steps=3 refract_base=0.48 refract_scale=1.55 distortion=62 normal=7.8 focus_blur=1.08 body=0.50 scene_light=3.0 rim=2.1 chroma=0.32");
}

fn dry_glass(world_api) {
    select(world_api, "Dry Glass");
    rg(world_api, "preset=debug debug=final seed=12801");
    rg(world_api, "spawn_rate=3 spawn_limit=80 radius_min=42 radius_max=110");
    rg(world_api, "gravity=1300 slip_rate=0.12 initial_spread=0.28 velocity_stretch=0.16 evaporate=22 collider=1.1");
    rg(world_api, "trails=false micro_enabled=false mist=false");
    rg(world_api, "blur=4 blur_steps=1 refract_base=0.70 refract_scale=2.4 distortion=72 normal=8.8 focus_blur=0.55 body=0.36 scene_light=2.2 rim=1.8 chroma=0.18");
}

fn debug_refraction(world_api) {
    select(world_api, "Debug Refraction");
    rg(world_api, "preset=optics_debug debug=final seed=99071");
    rg(world_api, "spawn_rate=2 spawn_limit=24 radius_min=80 radius_max=150");
    rg(world_api, "trails=true trail_density=0.50 trail_spread=1.10 streak=0.85 streak_len=1.65 trail_refract=0.50 trail_opacity=0.62 trail_dist_min=4 trail_dist_max=10");
    rg(world_api, "micro_enabled=false mist=false");
    rg(world_api, "blur=16 blur_steps=4 refract_base=1.05 refract_scale=4.0 distortion=96 normal=11 focus_blur=1.55 body=0.32 scene_light=3.4 rim=2.6 chroma=0.35 compose=hard");
}
