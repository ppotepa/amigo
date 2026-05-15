fn reset(world_api) {
    default_load(world_api);
    world_api.dev.log("RainGlass presets: press 1-9 or 0 for ten temporary scene presets.");
    world_api.dev.log("RainGlass debug: F1 final, F2 streak_map, F3 blurred_scene, C normals, V mask, B mist, R refraction, T raindrop_map.");
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
        debug_view(world_api, "Streak Map", "streak_map");
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

fn default_load(world_api) {
    select(world_api, "Cold Neon");
    rg(world_api, "preset=html_current_controls reference_mode=true compose=smoother debug=final seed=51499");
    rg(world_api, "spawn_rate=9 spawn_limit=820 radius_min=20 radius_max=68");
    rg(world_api, "gravity=2350 slip_rate=0.30 initial_spread=0.50 velocity_stretch=0.40 evaporate=11 collider=1.10");
    rg(world_api, "trails=true trail_density=0.34 trail_size_min=0.22 trail_size_max=0.52 trail_spread=0.98 streak=0.78 streak_len=1.48 trail_taper=0.62 trail_evaporate=12 trail_shrink_rate=0.62 trail_dist_min=7 trail_dist_max=16");
    rg(world_api, "micro_enabled=true micro=720 micro_min=5 micro_max=17");
    rg(world_api, "mist=true mist_opacity=0.28 mist_time=16 mist_color_strength=0.012 mist_accumulation=0.012 mist_blur_step=4");
    rg(world_api, "blur=0 blur_steps=0 drop_blur=2.4 refract_base=0.34 refract_scale=0.66 diffuse=0.16 specular=0.022 shininess=300 shadow=0.78 blend=0.50 body=0.50");
}

fn focus_foreground(world_api) {
    rg(world_api, "drop_blur=1.2 blur=3 blur_steps=3 blend=0.50 body=0.50");
}

fn reference(world_api) {
    select(world_api, "Reference / HTML 1:1");
    rg(world_api, "preset=html_current_controls reference_mode=true debug=final seed=121713");
    rg(world_api, "compose=smoother trails=true micro_enabled=true mist=true");
}

fn cinematic(world_api) {
    select(world_api, "HTML Cinematic Button");
    rg(world_api, "preset=html_cinematic_button reference_mode=true compose=smoother debug=final seed=121713");
    rg(world_api, "trails=true micro_enabled=true mist=true");
}

fn storm_wall(world_api) {
    select(world_api, "HTML Storm Button");
    rg(world_api, "preset=html_storm_button reference_mode=true compose=smoother debug=final seed=92131");
    rg(world_api, "trails=true micro_enabled=true mist=true");
}

fn long_streaks(world_api) {
    select(world_api, "Long Wet Streaks");
    rg(world_api, "preset=html_current_controls reference_mode=true compose=smoother debug=final seed=33017");
    rg(world_api, "spawn_rate=7 spawn_limit=760 radius_min=28 radius_max=82");
    rg(world_api, "gravity=2450 slip_rate=0.38 initial_spread=0.48 velocity_stretch=0.34 evaporate=10 collider=1.15");
    rg(world_api, "trails=true trail_density=0.36 trail_size_min=0.20 trail_size_max=0.48 trail_spread=1.25 streak=0.92 streak_len=2.15 trail_taper=0.72 trail_evaporate=18 trail_shrink_rate=0.36 trail_dist_min=6 trail_dist_max=14");
    rg(world_api, "micro_enabled=true micro=360 micro_min=5 micro_max=16");
    rg(world_api, "mist=true mist_opacity=0.24 mist_time=18 mist_color_strength=0.009 mist_accumulation=0.009 mist_blur_step=3");
    rg(world_api, "blur=2 blur_steps=2 refract_base=0.32 refract_scale=0.62 diffuse=0.20 specular=0.014 shininess=300 shadow=0.76");
}

fn fogged_glass(world_api) {
    select(world_api, "Fogged Glass");
    rg(world_api, "preset=html_current_controls reference_mode=true compose=smoother debug=final seed=88112");
    rg(world_api, "spawn_rate=4 spawn_limit=420 radius_min=18 radius_max=54");
    rg(world_api, "gravity=1900 slip_rate=0.18 initial_spread=0.42 velocity_stretch=0.24 evaporate=14 collider=1.05");
    rg(world_api, "trails=true trail_density=0.24 trail_size_min=0.18 trail_size_max=0.42 trail_spread=0.70 streak=0.36 streak_len=0.85 trail_taper=0.80 trail_evaporate=26 trail_shrink_rate=0.36 trail_dist_min=10 trail_dist_max=22");
    rg(world_api, "micro_enabled=true micro=520 micro_min=3 micro_max=12");
    rg(world_api, "mist=true mist_opacity=0.88 mist_time=20 mist_color_strength=0.030 mist_accumulation=0.030 mist_blur_step=6");
    rg(world_api, "blur=5 blur_steps=5 refract_base=0.30 refract_scale=0.62 diffuse=0.16 specular=0.006 shininess=340 shadow=0.72");
}

fn macro_lens(world_api) {
    select(world_api, "Large Lens Drops");
    rg(world_api, "preset=html_current_controls reference_mode=true compose=smoother debug=final seed=44901");
    rg(world_api, "spawn_rate=1.4 spawn_limit=42 radius_min=88 radius_max=176");
    rg(world_api, "gravity=1100 slip_rate=0.10 initial_spread=0.54 velocity_stretch=0.12 evaporate=5 collider=1.20");
    rg(world_api, "trails=false micro_enabled=false mist=false");
    rg(world_api, "blur=4 blur_steps=4 refract_base=0.52 refract_scale=0.76 diffuse=0.16 specular=0.026 shininess=256 shadow=0.78");
}

fn micro_beads(world_api) {
    select(world_api, "Balanced Rain Beads");
    rg(world_api, "preset=html_current_controls reference_mode=true compose=smoother debug=final seed=77219");
    rg(world_api, "spawn_rate=8 spawn_limit=900 radius_min=34 radius_max=92");
    rg(world_api, "gravity=2350 slip_rate=0.34 initial_spread=0.50 velocity_stretch=0.34 evaporate=11 collider=1.18");
    rg(world_api, "trails=true trail_density=0.38 trail_size_min=0.28 trail_size_max=0.62 trail_spread=1.22 streak=0.88 streak_len=1.70 trail_taper=0.72 trail_evaporate=18 trail_shrink_rate=0.36 trail_dist_min=7 trail_dist_max=17");
    rg(world_api, "micro_enabled=true micro=1150 micro_min=6 micro_max=22");
    rg(world_api, "mist=true mist_opacity=0.52 mist_time=14 mist_color_strength=0.024 mist_accumulation=0.024 mist_blur_step=4");
    rg(world_api, "blur=3 blur_steps=3 refract_base=0.38 refract_scale=0.74 diffuse=0.22 specular=0.020 shininess=300 shadow=0.78 chroma=0.12");
}

fn cold_neon(world_api) {
    select(world_api, "Cold Neon");
    rg(world_api, "preset=html_current_controls reference_mode=true compose=smoother debug=final seed=51499");
    rg(world_api, "spawn_rate=9 spawn_limit=820 radius_min=20 radius_max=68");
    rg(world_api, "gravity=2350 slip_rate=0.30 initial_spread=0.50 velocity_stretch=0.40 evaporate=11 collider=1.10");
    rg(world_api, "trails=true trail_density=0.34 trail_size_min=0.22 trail_size_max=0.52 trail_spread=0.98 streak=0.78 streak_len=1.48 trail_taper=0.62 trail_evaporate=12 trail_shrink_rate=0.62 trail_dist_min=7 trail_dist_max=16");
    rg(world_api, "micro_enabled=true micro=720 micro_min=5 micro_max=17");
    rg(world_api, "mist=true mist_opacity=0.28 mist_time=16 mist_color_strength=0.012 mist_accumulation=0.012 mist_blur_step=4");
    rg(world_api, "blur=3 blur_steps=3 refract_base=0.34 refract_scale=0.66 diffuse=0.16 specular=0.022 shininess=300 shadow=0.78 blend=0.92");
}

fn dry_glass(world_api) {
    select(world_api, "Clean Streak Glass");
    rg(world_api, "preset=html_current_controls reference_mode=true compose=smoother debug=final seed=12801");
    rg(world_api, "spawn_rate=4 spawn_limit=420 radius_min=28 radius_max=84");
    rg(world_api, "gravity=1550 slip_rate=0.18 initial_spread=0.30 velocity_stretch=0.20 evaporate=18 collider=1.12");
    rg(world_api, "trails=true trail_density=0.26 trail_size_min=0.18 trail_size_max=0.44 trail_spread=0.82 streak=0.62 streak_len=1.30 trail_taper=0.72 trail_evaporate=20 trail_shrink_rate=0.36 trail_dist_min=8 trail_dist_max=18");
    rg(world_api, "micro_enabled=true micro=420 micro_min=4 micro_max=14");
    rg(world_api, "mist=true mist_opacity=0.34 mist_time=18 mist_color_strength=0.014 mist_accumulation=0.014 mist_blur_step=4");
    rg(world_api, "blur=3 blur_steps=3 refract_base=0.48 refract_scale=0.82 diffuse=0.18 specular=0.024 shininess=260 shadow=0.78 chroma=0.10");
}

fn debug_refraction(world_api) {
    select(world_api, "Debug Refraction");
    rg(world_api, "preset=html_current_controls reference_mode=true compose=smoother debug=refraction seed=99071");
    rg(world_api, "spawn_rate=2 spawn_limit=36 radius_min=70 radius_max=140");
    rg(world_api, "gravity=900 slip_rate=0.06 initial_spread=0.40 velocity_stretch=0.10 evaporate=4 collider=1.20");
    rg(world_api, "trails=true trail_density=0.28 trail_size_min=0.20 trail_size_max=0.48 trail_spread=0.80 streak=0.55 streak_len=1.20 trail_taper=0.70 trail_evaporate=18 trail_shrink_rate=0.36 trail_dist_min=8 trail_dist_max=18");
    rg(world_api, "micro_enabled=false mist=false");
    rg(world_api, "blur=4 blur_steps=4 refract_base=0.62 refract_scale=0.94 diffuse=0.18 specular=0.030 shininess=256 shadow=0.78");
}
