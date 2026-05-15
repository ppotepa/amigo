import "mod:fx/math" as math;

fn rg(world_api, args) {
    world_api.postfx.set_rain_glass(args);
}

fn lerp(a, b, t) {
    a + (b - a) * t
}

fn smooth01(value) {
    let x = math::clamp01(value);
    x * x * (3.0 - 2.0 * x)
}

fn reset(world_api) {
    world_api.state.set_float("rain_lens.intro_t", 0.0);
    world_api.state.set_bool("rain_lens.menu_focus", false);
    world_api.state.set_float("rain_lens.focus_t", 0.0);
    let lens_distortion = world_api.state.get_float("camera_lens.distortion_px");
    let lens_chroma = world_api.state.get_float("camera_lens.chromatic_aberration");

    rg(world_api, "preset=html_current_controls reference_mode=true compose=smoother debug=final seed=51499");
    rg(world_api, "spawn_rate=0.9 spawn_limit=96 radius_min=16 radius_max=48");
    rg(world_api, "gravity=2050 slip_rate=0.22 initial_spread=0.44 velocity_stretch=0.26 evaporate=14 collider=1.08");
    rg(world_api, "trails=true trail_density=0.03 trail_size_min=0.16 trail_size_max=0.34 trail_spread=0.78 streak=0.08 streak_len=0.32 trail_taper=0.72 trail_evaporate=18 trail_shrink_rate=0.62 trail_dist_min=8 trail_dist_max=18");
    rg(world_api, "micro_enabled=true micro_droplets_per_second=0 micro_min=3 micro_max=9");
    rg(world_api, "mist=false mist_opacity=0 mist_time=16 mist_color_strength=0 mist_accumulation=0 mist_blur=0 mist_blur_step=0");
    rg(world_api, "blur=0.10 blur_steps=0 drop_blur=1.15 refract_base=0.24 refract_scale=0.46 opacity=0.18 body=0.42 blend=0.40 diffuse=0.10 specular=0.022 shininess=340 shadow=0.48 focus_blur=0.08 scene_light=0.08 light_tint=0.05 rim=0.16 chroma=" + lens_chroma.to_string() + " distortion=" + lens_distortion.to_string() + " scene_darken=0.08 blood_amount=0");
}

fn update_intro_lens(world_api, time) {
    let droplet_reveal = 0.38 + smooth01(time / 3.0) * 0.62;
    let rain_onset = smooth01((time - 12.0) / 2.0);
    let mist_reveal = smooth01((time - 14.0) / 2.0);
    let lens_distortion = world_api.state.get_float("camera_lens.distortion_px");
    let lens_chroma = world_api.state.get_float("camera_lens.chromatic_aberration");
    let lens_edge_warp = world_api.state.get_float("camera_lens.edge_warp");

    let drop_rate = lerp(0.9, 6.4, rain_onset) * droplet_reveal;
    let limit = lerp(96.0, 560.0, rain_onset) * droplet_reveal;
    let micro = lerp(0.0, 430.0, rain_onset);
    let trail_density = lerp(0.03, 0.24, rain_onset);
    let streak = lerp(0.08, 0.58, rain_onset);
    let streak_len = lerp(0.32, 1.18, rain_onset);
    let mist_opacity = lerp(0.0, 0.14, mist_reveal);
    let mist_color = lerp(0.0, 0.007, mist_reveal);
    let mist_accum = lerp(0.0, 0.007, mist_reveal);

    let background_blur = lerp(0.10, 1.65, mist_reveal);
    let drop_blur = lerp(1.15, 3.3, rain_onset);
    let focus_blur = lerp(0.08, 0.22, rain_onset);
    let scene_light = lerp(0.08, 0.16, rain_onset);
    let light_tint = lerp(0.05, 0.08, rain_onset);
    let scene_darken = lerp(0.08, 0.02, rain_onset);
    let opacity = lerp(0.18, 0.56, rain_onset) * droplet_reveal;
    let blend = lerp(0.40, 0.76, rain_onset);
    let chroma = lens_chroma + lerp(0.0, 0.035, rain_onset);
    let distortion = lens_distortion * (1.0 + lens_edge_warp * 0.22 + rain_onset * 0.18);

    rg(
        world_api,
        "spawn_rate=" + drop_rate.to_string()
        + " spawn_limit=" + limit.to_int().to_string()
        + " trail_density=" + trail_density.to_string()
        + " streak=" + streak.to_string()
        + " streak_len=" + streak_len.to_string()
        + " micro_droplets_per_second=" + micro.to_string()
        + " mist_opacity=" + mist_opacity.to_string()
        + " mist_color_strength=" + mist_color.to_string()
        + " mist_accumulation=" + mist_accum.to_string()
        + " mist_blur=8"
        + " mist_blur_step=4"
        + " blur=" + background_blur.to_string()
        + " blur_steps=3"
        + " drop_blur=" + drop_blur.to_string()
        + " focus_blur=" + focus_blur.to_string()
        + " scene_light=" + scene_light.to_string()
        + " light_tint=" + light_tint.to_string()
        + " scene_darken=" + scene_darken.to_string()
        + " opacity=" + opacity.to_string()
        + " blend=" + blend.to_string()
        + " refract_scale=0.50"
        + " chroma=" + chroma.to_string()
        + " distortion=" + distortion.to_string()
        + " blood_amount=0"
    );
}

fn begin_menu_lens(world_api) {
    world_api.state.set_bool("rain_lens.menu_focus", true);
    world_api.state.set_float("rain_lens.focus_t", 0.0);
}

fn update_menu_lens(world_api, dt) {
    if !world_api.state.get_bool("rain_lens.menu_focus") {
        return;
    }

    let t = world_api.state.get_float("rain_lens.focus_t") + dt;
    world_api.state.set_float("rain_lens.focus_t", t);

    let settle = smooth01(t / 2.4);
    let drop_rate = lerp(6.0, 4.8, settle);
    let micro = lerp(430.0, 320.0, settle);
    let background_blur = lerp(2.2, 2.8, settle);
    let drop_blur = lerp(3.6, 4.2, settle);
    let focus_blur = lerp(0.22, 0.42, settle);
    let scene_darken = lerp(0.14, 0.18, settle);
    let lens_distortion = world_api.state.get_float("camera_lens.distortion_px");
    let lens_chroma = world_api.state.get_float("camera_lens.chromatic_aberration");
    let lens_edge_warp = world_api.state.get_float("camera_lens.edge_warp");
    let distortion = lens_distortion * (1.0 + lens_edge_warp * 0.18);

    rg(
        world_api,
        "spawn_rate=" + drop_rate.to_string()
        + " micro_droplets_per_second=" + micro.to_string()
        + " blur=" + background_blur.to_string()
        + " blur_steps=4"
        + " drop_blur=" + drop_blur.to_string()
        + " focus_blur=" + focus_blur.to_string()
        + " opacity=0.58"
        + " blend=0.76"
        + " refract_scale=0.52"
        + " chroma=" + (lens_chroma + 0.020).to_string()
        + " distortion=" + distortion.to_string()
        + " scene_darken=" + scene_darken.to_string()
    );
}
