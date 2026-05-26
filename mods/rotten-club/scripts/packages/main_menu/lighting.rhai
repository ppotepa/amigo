import "mod:main_menu/ids" as ids;
import "mod:main_menu/math" as math;
import "mod:main_menu/timeline" as timeline;

fn reset(world_api) {
    let background = ids::background_entity();
    let lightning = world_api.light2d.get_group(ids::lightning_id());

    world_api.state.set_bool("intro.bar_lit", false);
    world_api.state.set_bool("intro.club_lit", false);
    world_api.state.set_bool("intro.pharmacy_lit", false);
    world_api.state.set_bool("intro.rain_started", false);
    world_api.state.set_bool("intro.lightning_fired", false);
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

fn update(world_api, time) {
    let background = ids::background_entity();
    let lightning = world_api.light2d.get_group(ids::lightning_id());
    let beacon_red = world_api.light2d.get_light("beacon-red");

    if time >= timeline::bar_light_start_s() { world_api.state.set_bool("intro.bar_lit", true); }
    if time >= timeline::club_light_start_s() { world_api.state.set_bool("intro.club_lit", true); }
    if time >= timeline::pharmacy_light_start_s() { world_api.state.set_bool("intro.pharmacy_lit", true); }
    if time >= timeline::rain_start_s() { world_api.state.set_bool("intro.rain_started", true); }
    if time >= timeline::lightning_start_s() { world_api.state.set_bool("intro.lightning_fired", true); }

    let skyline = math::ramp(time, timeline::skyline_focus_start_s(), 2.20);
    let club_focus = math::ramp(time, timeline::far_street_focus_start_s(), 10.0);
    let bar = math::ramp(time, timeline::bar_light_start_s(), 1.40);
    let club = math::ramp(time, timeline::club_light_start_s(), 1.60);
    let pharmacy = math::ramp(time, timeline::pharmacy_light_start_s(), 1.30);
    let rain = math::ramp(time, timeline::rain_start_s(), 2.20);
    let flash = math::pulse_long(time, timeline::lightning_start_s(), 0.06, 0.10, 0.42);

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

    let base = math::clamp01(bar * 0.018 + club * 0.026 + pharmacy * 0.012 + flash * 0.16);
    let skyline_light = math::clamp01(skyline * (0.38 + club_focus * 0.42) * skyline_mul + flash * 0.90);
    let bar_light = math::clamp01(bar * bar_mul + flash * 0.30);
    let lanterns = math::clamp01(bar * 0.86 * bar_mul + flash * 0.24);
    let club_light = math::clamp01(club * 0.72 * club_mul + flash * 0.26);
    let entry = math::clamp01(club * 0.42 * club_mul + flash * 0.18);
    let pharmacy_gate = if pharmacy_mul <= 0.0 { 0.0 } else { 1.0 };
    let pharmacy_shadow = pharmacy * 0.10 * pharmacy_gate;
    let pharmacy_light = math::clamp01(pharmacy_shadow + pharmacy * 0.78 * pharmacy_mul + flash * 0.18 * pharmacy_gate);
    let relief_edges = math::clamp01((skyline_light + rain * 0.20) * 0.82 + flash * 0.95);

    world_api.layered_image2d.set_base_opacity(background, base);
    world_api.layered_image2d.set_opacity(background, "skyline", skyline_light);
    world_api.layered_image2d.set_opacity(background, "bar_sign", bar_light);
    world_api.layered_image2d.set_opacity(background, "bar_lanterns", lanterns);
    world_api.layered_image2d.set_opacity(background, "club_sign", club_light);
    world_api.layered_image2d.set_opacity(background, "club_entry", entry);
    world_api.layered_image2d.set_opacity(background, "pharmacy_cross", pharmacy_light);
    world_api.layered_image2d.set_opacity(background, "rain_relief_edges", relief_edges);

    lightning.set_intensity(math::clamp01(flash + skyline * 0.018));
    beacon_red.set_intensity(math::clamp01(skyline * 0.08 * skyline_mul + club * 0.05 * club_mul));
    world_api.beacon2d.set_base_intensity(ids::beacon_a(), skyline * 1.20 * skyline_mul + bar * 0.65 * bar_mul + flash * 0.85);
    world_api.beacon2d.set_base_intensity(ids::beacon_b(), skyline * 1.00 * skyline_mul + club * 0.72 * club_mul + flash * 0.78);
    world_api.beacon2d.set_base_intensity(ids::beacon_c(), skyline * 1.10 + pharmacy * 0.68 * pharmacy_gate + flash * 0.82);
    world_api.beacon2d.set_base_intensity(ids::beacon_d_sync_1(), skyline * 0.90 * skyline_mul + club * 0.54 * club_mul + flash * 0.70);
    world_api.beacon2d.set_base_intensity(ids::beacon_d_sync_2(), skyline * 0.90 * skyline_mul + club * 0.54 * club_mul + flash * 0.70);

    [skyline_light, math::maxf(club_light, entry), math::maxf(bar_light, lanterns)]
}
