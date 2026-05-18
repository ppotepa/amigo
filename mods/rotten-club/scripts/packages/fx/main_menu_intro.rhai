import "mod:fx/ids" as ids;
import "mod:fx/sequence" as seq;

fn final_title_visibility() {
    1.0
}

fn reset(world_api) {
    let background = ids::background_entity();
    let lightning = world_api.light2d.get_group(ids::lightning_id());

    world_api.state.set_float("intro.t", 0.0);
    world_api.state.set_bool("intro.focus.skyline_started", false);
    world_api.state.set_bool("intro.focus.far_street_started", false);
    world_api.state.set_bool("intro.focus.club_started", false);
    world_api.state.set_bool("intro.focus.title_started", false);
    world_api.state.set_bool("intro.focus_hunt.near_started", false);
    world_api.state.set_bool("intro.focus_hunt.far_started", false);
    world_api.state.set_bool("intro.focus_hunt.locked", false);
    world_api.state.set_bool("intro.bar_lit", false);
    world_api.state.set_bool("intro.club_lit", false);
    world_api.state.set_bool("intro.pharmacy_lit", false);
    world_api.state.set_bool("intro.rain_started", false);
    world_api.state.set_bool("intro.lightning_fired", false);
    world_api.state.set_bool("intro.title_reveal_started", false);
    world_api.state.set_bool("intro.title_shown", false);
    world_api.state.set_bool("intro.complete", false);
    world_api.state.set_float("intro.title_alpha", 0.0);
    world_api.state.set_float("intro.rain_intensity", 0.0);
    world_api.state.set_float("intro.lightning_intensity", 0.0);
    world_api.state.set_float("bar.next_flicker", 0.0);
    world_api.state.set_float("bar.mul", 1.0);
    world_api.state.set_float("club.next_flicker", 0.0);
    world_api.state.set_float("club.mul", 1.0);
    world_api.state.set_float("skyline.next_flicker", 0.0);
    world_api.state.set_float("skyline.mul", 1.0);
    world_api.state.set_float("pharmacy.next_fault", 12.0);
    world_api.state.set_float("pharmacy.mul", 0.0);

    world_api.entities.hide(ids::title_entity());
    world_api.render2d.get_layer(ids::title_layer()).set_opacity(0.0);

    world_api.layered_image2d.set_base_opacity(background, 0.0);
    world_api.layered_image2d.set_opacity(background, "skyline", 0.0);
    world_api.layered_image2d.set_opacity(background, "club_sign", 0.0);
    world_api.layered_image2d.set_opacity(background, "club_entry", 0.0);
    world_api.layered_image2d.set_opacity(background, "bar_sign", 0.0);
    world_api.layered_image2d.set_opacity(background, "bar_lanterns", 0.0);
    world_api.layered_image2d.set_opacity(background, "pharmacy_cross", 0.0);
    world_api.layered_image2d.set_opacity(background, "rain_relief_edges", 0.0);

    lightning.set_intensity(0.0);
    world_api.light2d.get_light("beacon-red").set_intensity(0.0);
    world_api.beacon2d.set_base_intensity(ids::beacon_a(), 0.0);
    world_api.beacon2d.set_base_intensity(ids::beacon_b(), 0.0);
    world_api.beacon2d.set_base_intensity(ids::beacon_c(), 0.0);
    world_api.beacon2d.set_base_intensity(ids::beacon_d_sync_1(), 0.0);
    world_api.beacon2d.set_base_intensity(ids::beacon_d_sync_2(), 0.0);
    world_api.camera.set_main_aperture_f_stop(5.6);
}

fn update_focus(world_api, time) {
    set_focus_once(world_api, "intro.focus.skyline_started", ids::background_layer(), 0.0);
    update_initial_focus_hunt(world_api, time);

    if time >= 5.00 {
        set_focus_once(world_api, "intro.focus.far_street_started", ids::rain_10m_layer(), 5.0);
    }

    if time >= 10.00 {
        set_focus_once(world_api, "intro.focus.club_started", ids::rain_5m_layer(), 5.0);
    }

    if time >= 15.00 {
        set_focus_once(world_api, "intro.focus.title_started", ids::title_focus_target(), 2.0);
    }
}

fn update_initial_focus_hunt(world_api, time) {
    if time >= 2.40 {
        set_focus_once(world_api, "intro.focus_hunt.locked", ids::background_layer(), 0.45);
        return;
    }

    if time >= 0.55 {
        set_focus_once(world_api, "intro.focus_hunt.near_started", ids::rain_10m_layer(), 0.55);
    }

    if time >= 1.30 {
        set_focus_once(world_api, "intro.focus_hunt.far_started", ids::background_layer(), 0.70);
    }
}

fn set_focus_once(world_api, key, selector, seconds) {
    if world_api.state.get_bool(key) {
        return;
    }

    if world_api.camera.focus_main_over(selector, seconds) {
        world_api.state.set_bool(key, true);
    }
}

fn update_pharmacy_fault(world_api, time, active) {
    if !active {
        world_api.state.set_float("pharmacy.mul", 0.0);
        return;
    }

    if time >= world_api.state.get_float("pharmacy.next_fault") {
        let roll = world_api.random.range(0.0, 1.0);
        if roll < 0.42 {
            world_api.state.set_float("pharmacy.mul", 0.0);
            world_api.state.set_float("pharmacy.next_fault", time + world_api.random.range(0.16, 0.48));
            return;
        }
        if roll < 0.66 {
            world_api.state.set_float("pharmacy.mul", 0.10);
            world_api.state.set_float("pharmacy.next_fault", time + world_api.random.range(0.12, 0.34));
            return;
        }
        if roll < 0.82 {
            world_api.state.set_float("pharmacy.mul", 0.15);
            world_api.state.set_float("pharmacy.next_fault", time + world_api.random.range(0.10, 0.28));
            return;
        }
        if roll < 0.93 {
            world_api.state.set_float("pharmacy.mul", 0.25);
            world_api.state.set_float("pharmacy.next_fault", time + world_api.random.range(0.08, 0.22));
            return;
        }
        if roll < 0.985 {
            world_api.state.set_float("pharmacy.mul", 0.50);
            world_api.state.set_float("pharmacy.next_fault", time + world_api.random.range(0.05, 0.14));
            return;
        }

        world_api.state.set_float("pharmacy.mul", 1.0);
        world_api.state.set_float("pharmacy.next_fault", time + world_api.random.range(0.035, 0.08));
    }
}

fn update_soft_flicker(world_api, time, prefix, active, min_value, max_value, min_delay, max_delay) {
    if !active {
        world_api.state.set_float(prefix + ".mul", 1.0);
        return;
    }

    if time >= world_api.state.get_float(prefix + ".next_flicker") {
        world_api.state.set_float(prefix + ".mul", world_api.random.range(min_value, max_value));
        world_api.state.set_float(prefix + ".next_flicker", time + world_api.random.range(min_delay, max_delay));
    }
}

fn update_title(world_api, time) {
    if time >= 17.00 && seq::once(world_api, "intro.title_reveal_started") {
        world_api.entities.show(ids::title_entity());
        world_api.state.set_bool("intro.title_shown", true);
    }

    let reveal = seq::ramp(time, 17.00, 2.0);
    let alpha = seq::mix(0.0, final_title_visibility(), reveal);
    world_api.state.set_float("intro.title_alpha", alpha);
    world_api.render2d.get_layer(ids::title_layer()).set_opacity(alpha);

    if time >= 15.00 {
        let t = seq::ramp(time, 15.00, 2.0);
        world_api.camera.set_main_aperture_f_stop(seq::mix(4.0, 1.6, t));
    }
}

fn update_lights(world_api, time) {
    let background = ids::background_entity();
    let lightning = world_api.light2d.get_group(ids::lightning_id());
    let beacon_red = world_api.light2d.get_light("beacon-red");

    if time >= 5.60 { world_api.state.set_bool("intro.bar_lit", true); }
    if time >= 7.20 { world_api.state.set_bool("intro.club_lit", true); }
    if time >= 9.00 { world_api.state.set_bool("intro.pharmacy_lit", true); }
    if time >= 11.00 { world_api.state.set_bool("intro.rain_started", true); }
    if time >= 14.20 { world_api.state.set_bool("intro.lightning_fired", true); }

    let skyline = seq::ramp(time, 2.40, 2.20);
    let club_focus = seq::ramp(time, 5.00, 10.0);
    let bar = seq::ramp(time, 5.60, 1.40);
    let club = seq::ramp(time, 7.20, 1.60);
    let pharmacy = seq::ramp(time, 9.00, 1.30);
    let rain = seq::ramp(time, 11.00, 2.20);
    let flash = seq::pulse_long(time, 14.20, 0.06, 0.10, 0.42);
    update_soft_flicker(world_api, time, "skyline", skyline > 0.01, 0.96, 1.03, 0.38, 0.95);
    update_soft_flicker(world_api, time, "bar", bar > 0.01, 0.82, 1.10, 0.18, 0.58);
    update_soft_flicker(world_api, time, "club", club > 0.01, 0.96, 1.04, 0.08, 0.32);
    update_pharmacy_fault(world_api, time, pharmacy > 0.01);
    let skyline_mul = world_api.state.get_float("skyline.mul");
    let bar_mul = world_api.state.get_float("bar.mul");
    let club_mul = world_api.state.get_float("club.mul");
    let pharmacy_mul = world_api.state.get_float("pharmacy.mul");

    world_api.state.set_float("intro.rain_intensity", rain);
    world_api.state.set_float("intro.lightning_intensity", flash);

    let base = seq::clamp01(bar * 0.018 + club * 0.026 + pharmacy * 0.012 + flash * 0.16);
    let skyline_light = seq::clamp01(skyline * (0.38 + club_focus * 0.42) * skyline_mul + flash * 0.90);
    let bar_light = seq::clamp01(bar * bar_mul + flash * 0.30);
    let lanterns = seq::clamp01(bar * 0.86 * bar_mul + flash * 0.24);
    let club_light = seq::clamp01(club * 0.72 * club_mul + flash * 0.26);
    let entry = seq::clamp01(club * 0.42 * club_mul + flash * 0.18);
    let pharmacy_gate = if pharmacy_mul <= 0.0 { 0.0 } else { 1.0 };
    let pharmacy_shadow = pharmacy * 0.10 * pharmacy_gate;
    let pharmacy_light = seq::clamp01(pharmacy_shadow + pharmacy * 0.78 * pharmacy_mul + flash * 0.18 * pharmacy_gate);
    let relief_edges = seq::clamp01((skyline_light + rain * 0.20) * 0.82 + flash * 0.95);

    world_api.layered_image2d.set_base_opacity(background, base);
    world_api.layered_image2d.set_opacity(background, "skyline", skyline_light);
    world_api.layered_image2d.set_opacity(background, "bar_sign", bar_light);
    world_api.layered_image2d.set_opacity(background, "bar_lanterns", lanterns);
    world_api.layered_image2d.set_opacity(background, "club_sign", club_light);
    world_api.layered_image2d.set_opacity(background, "club_entry", entry);
    world_api.layered_image2d.set_opacity(background, "pharmacy_cross", pharmacy_light);
    world_api.layered_image2d.set_opacity(background, "rain_relief_edges", relief_edges);

    lightning.set_intensity(seq::clamp01(flash + skyline * 0.018));
    beacon_red.set_intensity(seq::clamp01(skyline * 0.08 * skyline_mul + club * 0.05 * club_mul));
    world_api.beacon2d.set_base_intensity(ids::beacon_a(), skyline * 1.20 * skyline_mul + bar * 0.65 * bar_mul + flash * 0.85);
    world_api.beacon2d.set_base_intensity(ids::beacon_b(), skyline * 1.00 * skyline_mul + club * 0.72 * club_mul + flash * 0.78);
    world_api.beacon2d.set_base_intensity(ids::beacon_c(), skyline * 1.10 + pharmacy * 0.68 * pharmacy_gate + flash * 0.82);
    world_api.beacon2d.set_base_intensity(ids::beacon_d_sync_1(), skyline * 0.90 * skyline_mul + club * 0.54 * club_mul + flash * 0.70);
    world_api.beacon2d.set_base_intensity(ids::beacon_d_sync_2(), skyline * 0.90 * skyline_mul + club * 0.54 * club_mul + flash * 0.70);

    [skyline_light, seq::maxf(club_light, entry), seq::maxf(bar_light, lanterns)]
}

fn update(world_api, time, dt) {
    update_focus(world_api, time);
    let lights = update_lights(world_api, time);
    update_title(world_api, time);

    if time >= 19.00 {
        world_api.state.set_bool("intro.complete", true);
    }

    lights
}

fn complete(world_api) {
    world_api.state.get_bool("intro.complete")
}
