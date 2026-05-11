import "mod:tar_fx/ids" as ids;
import "mod:tar_fx/math" as math;

fn set_layer_opacities(world_api, background, value) {
    world_api.layered_image2d.set_opacity(background, "skyline", value);
    world_api.layered_image2d.set_opacity(background, "skyline_blur", value);
    world_api.layered_image2d.set_opacity(background, "aircraft_beacons", value);
    world_api.layered_image2d.set_opacity(background, "aircraft_beacons_blur", value);
    world_api.layered_image2d.set_opacity(background, "club_sign", value);
    world_api.layered_image2d.set_opacity(background, "club_sign_blur", value);
    world_api.layered_image2d.set_opacity(background, "club_entry", value);
    world_api.layered_image2d.set_opacity(background, "club_entry_blur", value);
    world_api.layered_image2d.set_opacity(background, "bar_sign", value);
    world_api.layered_image2d.set_opacity(background, "bar_sign_blur", value);
    world_api.layered_image2d.set_opacity(background, "bar_lanterns", value);
    world_api.layered_image2d.set_opacity(background, "bar_lanterns_blur", value);
    world_api.layered_image2d.set_opacity(background, "pharmacy_cross", value);
    world_api.layered_image2d.set_opacity(background, "pharmacy_cross_blur", value);
    world_api.layered_image2d.set_opacity(background, "rain_relief_edges", value);
    world_api.layered_image2d.set_opacity(background, "puddle_reflection_soft", value);
    world_api.layered_image2d.set_opacity(background, "puddle_reflection_highlights", value);
}


fn reset(world_api) {
    world_api.state.set_float("intro.t", 0.0);
    world_api.state.set_bool("intro.prompt_shown", false);
    world_api.state.set_bool("intro.menu_active", false);
    world_api.state.set_float("neon.next_flicker", 0.0);
    world_api.state.set_float("neon.mul", 0.0);
    world_api.state.set_float("pharmacy.next_fault", 8.0);
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
        world_api.state.set_float("pharmacy.mul", 1.0);
        return;
    }

    if time >= world_api.state.get_float("pharmacy.next_fault") {
        let roll = world_api.random.range(0.0, 1.0);
        if roll < 0.32 {
            world_api.state.set_float("pharmacy.mul", 0.0);
            world_api.state.set_float("pharmacy.next_fault", time + world_api.random.range(0.04, 0.12));
            return;
        }
        if roll < 0.58 {
            world_api.state.set_float("pharmacy.mul", world_api.random.range(0.12, 0.38));
            world_api.state.set_float("pharmacy.next_fault", time + world_api.random.range(0.05, 0.20));
            return;
        }
        world_api.state.set_float("pharmacy.mul", world_api.random.range(0.72, 1.0));
        world_api.state.set_float("pharmacy.next_fault", time + world_api.random.range(0.18, 0.75));
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
    time >= 10.0
}

fn update(world_api, time) {
    let background = ids::background_entity();
    let lightning = world_api.light2d.get_group(ids::lightning_id());
    let done = intro_done(time);
    update_neon_flicker(world_api, time, done);
    update_pharmacy_fault(world_api, time, done);

    let flash = lightning_flash(world_api, time);
    let neon_mul = world_api.state.get_float("neon.mul");
    let pharmacy_mul = world_api.state.get_float("pharmacy.mul");
    lightning.set_intensity(flash);

    let background_reveal = math::fade(time, 0.0, 4.0);
    let bar_reveal = math::fade(time, 4.6, 1.0);
    let neon_reveal = math::fade(time, 6.2, 1.8);
    let pharmacy_reveal = math::fade(time, 8.4, 1.6);

    let base = math::clamp01(flash * 0.88);
    let skyline = math::clamp01(background_reveal + flash * 0.92);
    let skyline_blur = math::clamp01(skyline * math::wave(time, 0.38, 0.4, 0.62, 1.05) + flash * 0.85);
    let beacons = math::clamp01(background_reveal * 0.45 + flash * 0.46);
    let beacons_blur = math::clamp01(beacons * math::wave(time, 0.72, 1.8, 0.30, 0.58) + flash * 0.36);
    let bar = math::clamp01(bar_reveal + flash * 0.32);
    let bar_blur = math::clamp01(bar * math::wave(time, 0.30, 2.6, 0.08, 0.18) + flash * 0.08);
    let lanterns = math::clamp01(bar_reveal * 0.85 + flash * 0.26);
    let lanterns_blur = math::clamp01(lanterns * math::wave(time, 0.34, 3.4, 0.10, 0.22) + flash * 0.08);
    let club_false_start = math::fade(time, 5.45, 0.16) * (1.0 - math::fade(time, 5.70, 0.16)) * 0.32;
    let club = math::clamp01((neon_reveal * 0.62 + club_false_start) * neon_mul + flash * 0.26);
    let club_blur = math::clamp01(club * math::wave(time, 0.64, 0.9, 0.28, 0.58) + flash * 0.20);
    let entry = math::clamp01(neon_reveal * 0.40 * neon_mul + flash * 0.20);
    let entry_blur = math::clamp01(entry * math::wave(time, 0.58, 2.0, 0.20, 0.44) + flash * 0.14);
    let pharmacy = math::clamp01(pharmacy_reveal * 0.68 * pharmacy_mul + flash * 0.28);
    let pharmacy_blur = math::clamp01(pharmacy * math::wave(time, 0.52, 4.1, 0.22, 0.52) + flash * 0.18);
    let relief_edges = math::clamp01(skyline * 0.72 + skyline_blur * 0.28 + flash * 0.95);
    let puddle_soft = math::clamp01(base * 0.22 + skyline_blur * 0.20 + bar_blur * 0.10 + flash * 0.32);
    let puddle_highlights = math::clamp01(puddle_soft * 0.42 + club_blur * 0.06 + pharmacy_blur * 0.06 + flash * 0.18);

    world_api.layered_image2d.set_base_opacity(background, base);
    world_api.layered_image2d.set_opacity(background, "skyline", skyline);
    world_api.layered_image2d.set_opacity(background, "skyline_blur", skyline_blur);
    world_api.layered_image2d.set_opacity(background, "aircraft_beacons", beacons);
    world_api.layered_image2d.set_opacity(background, "aircraft_beacons_blur", beacons_blur);
    world_api.layered_image2d.set_opacity(background, "club_sign", club);
    world_api.layered_image2d.set_opacity(background, "club_sign_blur", club_blur);
    world_api.layered_image2d.set_opacity(background, "club_entry", entry);
    world_api.layered_image2d.set_opacity(background, "club_entry_blur", entry_blur);
    world_api.layered_image2d.set_opacity(background, "bar_sign", bar);
    world_api.layered_image2d.set_opacity(background, "bar_sign_blur", bar_blur);
    world_api.layered_image2d.set_opacity(background, "bar_lanterns", lanterns);
    world_api.layered_image2d.set_opacity(background, "bar_lanterns_blur", lanterns_blur);
    world_api.layered_image2d.set_opacity(background, "pharmacy_cross", pharmacy);
    world_api.layered_image2d.set_opacity(background, "pharmacy_cross_blur", pharmacy_blur);
    world_api.layered_image2d.set_opacity(background, "rain_relief_edges", relief_edges);
    world_api.layered_image2d.set_opacity(background, "puddle_reflection_soft", puddle_soft);
    world_api.layered_image2d.set_opacity(background, "puddle_reflection_highlights", puddle_highlights);

    [skyline, math::maxf(club, entry), math::maxf(bar, lanterns)]
}
