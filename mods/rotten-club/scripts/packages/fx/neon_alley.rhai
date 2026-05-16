import "mod:fx/ids" as ids;
import "mod:fx/math" as math;

fn set_layer_opacities(world_api, background, value) {
    world_api.layered_image2d.set_opacity(background, "skyline", value);
    world_api.layered_image2d.set_opacity(background, "club_sign", value);
    world_api.layered_image2d.set_opacity(background, "club_entry", value);
    world_api.layered_image2d.set_opacity(background, "bar_sign", value);
    world_api.layered_image2d.set_opacity(background, "bar_lanterns", value);
    world_api.layered_image2d.set_opacity(background, "pharmacy_cross", value);
    world_api.layered_image2d.set_opacity(background, "rain_relief_edges", value);
}


fn reset(world_api) {
    world_api.state.set_float("intro.t", 0.0);
    world_api.state.set_bool("intro.prompt_shown", false);
    world_api.state.set_bool("intro.menu_active", false);
    world_api.state.set_float("neon.next_flicker", 0.0);
    world_api.state.set_float("neon.mul", 0.0);
    world_api.state.set_float("pharmacy.next_fault", 12.0);
    world_api.state.set_float("pharmacy.mul", 0.0);
    world_api.state.set_float("lightning.next", world_api.random.range(5.0, 15.0));
    world_api.state.set_bool("lightning.major", false);
    world_api.state.set_float("lightning.start", -100.0);
    world_api.state.set_float("lightning.until", 0.0);
    world_api.state.set_float("lightning.duration", 0.1);
    world_api.state.set_float("lightning.pulse0.center", 0.08);
    world_api.state.set_float("lightning.pulse0.width", 0.06);
    world_api.state.set_float("lightning.pulse0.strength", 0.0);
    world_api.state.set_float("lightning.pulse1.center", 0.38);
    world_api.state.set_float("lightning.pulse1.width", 0.08);
    world_api.state.set_float("lightning.pulse1.strength", 0.0);
    world_api.state.set_float("lightning.pulse2.center", 0.72);
    world_api.state.set_float("lightning.pulse2.width", 0.10);
    world_api.state.set_float("lightning.pulse2.strength", 0.0);
    world_api.beacon2d.set_base_intensity(ids::beacon_a(), 0.0);
    world_api.beacon2d.set_base_intensity(ids::beacon_b(), 0.0);
    world_api.beacon2d.set_base_intensity(ids::beacon_c(), 0.0);
    world_api.beacon2d.set_base_intensity(ids::beacon_d_sync_1(), 0.0);
    world_api.beacon2d.set_base_intensity(ids::beacon_d_sync_2(), 0.0);
    world_api.light2d.get_light("beacon-red").set_intensity(0.0);
}


fn update_neon_flicker(world_api, time, active) {
    if !active {
        world_api.state.set_float("neon.mul", 1.0);
        return;
    }

    if time >= world_api.state.get_float("neon.next_flicker") {
        world_api.state.set_float("neon.mul", world_api.random.range(0.78, 1.05));
        world_api.state.set_float("neon.next_flicker", time + world_api.random.range(0.06, 0.28));
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


fn lightning_flash(world_api, time) {
    if time >= world_api.state.get_float("lightning.next") {
        let major = world_api.random.range(0.0, 1.0) < 0.16;
        let duration = if major {
            world_api.random.range(1.05, 1.35)
        } else {
            world_api.random.range(0.10, 0.42)
        };
        world_api.state.set_bool("lightning.major", major);
        world_api.state.set_float("lightning.duration", duration);
        world_api.state.set_float("lightning.start", time);
        world_api.state.set_float("lightning.until", time + duration);
        if major {
            world_api.state.set_float("lightning.pulse0.center", 0.42);
            world_api.state.set_float("lightning.pulse0.width", 0.46);
            world_api.state.set_float("lightning.pulse0.strength", world_api.random.range(1.65, 2.0));
            world_api.state.set_float("lightning.pulse1.center", 0.18);
            world_api.state.set_float("lightning.pulse1.width", 0.10);
            world_api.state.set_float("lightning.pulse1.strength", world_api.random.range(0.65, 1.0));
            world_api.state.set_float("lightning.pulse2.center", 0.82);
            world_api.state.set_float("lightning.pulse2.width", 0.16);
            world_api.state.set_float("lightning.pulse2.strength", world_api.random.range(0.45, 0.85));
        } else {
            world_api.state.set_float("lightning.pulse0.center", world_api.random.range(0.04, 0.14));
            world_api.state.set_float("lightning.pulse0.width", world_api.random.range(0.035, 0.085));
            world_api.state.set_float("lightning.pulse0.strength", world_api.random.range(0.65, 1.0));
            world_api.state.set_float("lightning.pulse1.center", world_api.random.range(0.28, 0.56));
            world_api.state.set_float("lightning.pulse1.width", world_api.random.range(0.045, 0.10));
            if world_api.random.range(0.0, 1.0) < 0.72 {
                world_api.state.set_float("lightning.pulse1.strength", world_api.random.range(0.35, 0.78));
            } else {
                world_api.state.set_float("lightning.pulse1.strength", 0.0);
            }
            world_api.state.set_float("lightning.pulse2.center", world_api.random.range(0.58, 0.92));
            world_api.state.set_float("lightning.pulse2.width", world_api.random.range(0.055, 0.12));
            if world_api.random.range(0.0, 1.0) < 0.44 {
                world_api.state.set_float("lightning.pulse2.strength", world_api.random.range(0.18, 0.55));
            } else {
                world_api.state.set_float("lightning.pulse2.strength", 0.0);
            }
        }
        world_api.state.set_float("lightning.next", time + duration + world_api.random.range(5.0, 17.0));
    }

    let end_time = world_api.state.get_float("lightning.until");
    if time >= end_time {
        return 0.0;
    }

    let duration = world_api.state.get_float("lightning.duration");
    let phase = math::clamp01((time - world_api.state.get_float("lightning.start")) / duration);
    let pulse0 = math::lightning_pulse(
        phase,
        world_api.state.get_float("lightning.pulse0.center"),
        world_api.state.get_float("lightning.pulse0.width"),
        world_api.state.get_float("lightning.pulse0.strength")
    );
    let pulse1 = math::lightning_pulse(
        phase,
        world_api.state.get_float("lightning.pulse1.center"),
        world_api.state.get_float("lightning.pulse1.width"),
        world_api.state.get_float("lightning.pulse1.strength")
    );
    let pulse2 = math::lightning_pulse(
        phase,
        world_api.state.get_float("lightning.pulse2.center"),
        world_api.state.get_float("lightning.pulse2.width"),
        world_api.state.get_float("lightning.pulse2.strength")
    );

    let flash = math::maxf(pulse0, math::maxf(pulse1, pulse2));
    if world_api.state.get_bool("lightning.major") {
        let hold = 1.0 - math::absf(phase - 0.50) / 0.42;
        if hold > 0.0 {
            return math::maxf(flash, hold * 1.75);
        }
    }

    flash
}


fn intro_done(time) {
    time >= 15.0
}

fn update(world_api, time) {
    let background = ids::background_entity();
    let lightning = world_api.light2d.get_group(ids::lightning_id());
    let beacon_red = world_api.light2d.get_light("beacon-red");
    let done = intro_done(time);
    update_neon_flicker(world_api, time, done);

    let skyline_reveal = math::fade(time, 2.0, 3.0);
    let focus_reveal = math::fade(time, 5.0, 3.0);
    let bar_reveal = math::fade(time, 6.0, 2.0);
    let neon_reveal = math::fade(time, 10.0, 2.0);
    let beacon_reveal = skyline_reveal;
    let pharmacy_reveal = math::fade(time, 12.0, 1.0);
    update_pharmacy_fault(world_api, time, pharmacy_reveal > 0.01);

    let flash = lightning_flash(world_api, time);
    let neon_mul = world_api.state.get_float("neon.mul");
    let pharmacy_mul = world_api.state.get_float("pharmacy.mul");

    let cinematic_droplet_light = (1.0 - skyline_reveal) * 0.035 + skyline_reveal * 0.018;
    lightning.set_intensity(math::clamp01(flash + cinematic_droplet_light));
    beacon_red.set_intensity(math::clamp01(beacon_reveal * 0.13));

    world_api.beacon2d.set_base_intensity(ids::beacon_a(), beacon_reveal * 2.20);
    world_api.beacon2d.set_base_intensity(ids::beacon_b(), beacon_reveal * 1.95);
    world_api.beacon2d.set_base_intensity(ids::beacon_c(), beacon_reveal * 2.08);
    world_api.beacon2d.set_base_intensity(ids::beacon_d_sync_1(), beacon_reveal * 2.02);
    world_api.beacon2d.set_base_intensity(ids::beacon_d_sync_2(), beacon_reveal * 2.02);

    let base = math::clamp01(
        bar_reveal * 0.025
            + neon_reveal * 0.035
            + pharmacy_reveal * 0.015
            + flash * 0.18
    );
    let skyline = math::clamp01(skyline_reveal * (0.34 + focus_reveal * 0.66) + flash * 0.92);
    let bar = math::clamp01(bar_reveal + flash * 0.32);
    let lanterns = math::clamp01(bar_reveal * 0.85 + flash * 0.26);
    let club_false_start = 0.0;
    let club = math::clamp01((neon_reveal * 0.62 + club_false_start) * neon_mul + flash * 0.26);
    let entry = math::clamp01(neon_reveal * 0.40 * neon_mul + flash * 0.20);
    let pharmacy_gate = if pharmacy_mul <= 0.0 { 0.0 } else { 1.0 };
    let pharmacy_shadow = pharmacy_reveal * 0.10 * pharmacy_gate;
    let pharmacy = math::clamp01(pharmacy_shadow + pharmacy_reveal * 0.78 * pharmacy_mul + flash * 0.18 * pharmacy_gate);
    let relief_edges = math::clamp01(skyline * 0.82 + flash * 0.95);

    world_api.layered_image2d.set_base_opacity(background, base);
    world_api.layered_image2d.set_opacity(background, "skyline", skyline);
    world_api.layered_image2d.set_opacity(background, "club_sign", club);
    world_api.layered_image2d.set_opacity(background, "club_entry", entry);
    world_api.layered_image2d.set_opacity(background, "bar_sign", bar);
    world_api.layered_image2d.set_opacity(background, "bar_lanterns", lanterns);
    world_api.layered_image2d.set_opacity(background, "pharmacy_cross", pharmacy);
    world_api.layered_image2d.set_opacity(background, "rain_relief_edges", relief_edges);

    [skyline, math::maxf(club, entry), math::maxf(bar, lanterns)]
}
