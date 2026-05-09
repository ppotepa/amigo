fn background_entity() { "they-are-rotten-main-menu-background" }
fn rain_far_entity() { "they-are-rotten-main-menu-rain-far" }
fn rain_mid_entity() { "they-are-rotten-main-menu-rain-mid" }
fn rain_near_entity() { "they-are-rotten-main-menu-rain-near" }
fn menu_entity() { "they-are-rotten-main-menu" }
fn press_any_key_entity() { "they-are-rotten-main-menu-press-any-key" }
fn lightning_id() { "lightning" }

fn seed_menu_state(world_api) {
    world_api.state.set_int("menu_index", 0);

    world_api.state.set_string("menu_color.0", "#39D7FFFF");
    world_api.state.set_string("menu_color.1", "#7D8796FF");
    world_api.state.set_string("menu_color.2", "#7D8796FF");

    world_api.state.set_string("menu_status", "READY");
    world_api.state.set_string("menu_status_color", "#7D8796FF");
}

fn clamp01(value) {
    if value < 0.0 {
        return 0.0;
    }
    if value > 1.0 {
        return 1.0;
    }
    value
}

fn absf(value) {
    if value < 0.0 {
        return -value;
    }
    value
}

fn maxf(left, right) {
    if left > right {
        return left;
    }
    right
}

fn wave(time, speed, phase, min_value, max_value) {
    let t = (time * speed + phase).sin() * 0.5 + 0.5;
    min_value + (max_value - min_value) * t
}

fn fade(time, start, duration) {
    let x = clamp01((time - start) / duration);
    x * x * (3.0 - 2.0 * x)
}

fn lightning_pulse(phase, center, width, strength) {
    let shape = 1.0 - absf(phase - center) / width;
    if shape <= 0.0 {
        return 0.0;
    }
    shape * shape * strength
}

fn set_intro_layer_opacities(world_api, background, value) {
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
}

fn reset_intro_state(world_api) {
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

fn reset_rain_state(world_api) {
    world_api.state.set_float("rain.next_gust", world_api.random.range(3.5, 8.5));
    world_api.state.set_float("rain.gust_start", -100.0);
    world_api.state.set_float("rain.gust_until", -100.0);
    world_api.state.set_float("rain.gust_duration", 1.0);
    world_api.state.set_float("rain.gust_strength", 0.0);
    world_api.state.set_float("rain.gust_dir", 1.0);

    world_api.particles.set_intensity(rain_far_entity(), 0.0);
    world_api.particles.set_intensity(rain_mid_entity(), 0.0);
    world_api.particles.set_intensity(rain_near_entity(), 0.0);
    world_api.particles.set_wind(rain_far_entity(), 68.0, -320.0, 0.12);
    world_api.particles.set_wind(rain_mid_entity(), 112.0, -500.0, 0.16);
    world_api.particles.set_wind(rain_near_entity(), 150.0, -680.0, 0.18);
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
    let phase = clamp01((time - world_api.state.get_float("lightning.start")) / duration);
    let pulse0 = lightning_pulse(
        phase,
        world_api.state.get_float("lightning.pulse0.center"),
        world_api.state.get_float("lightning.pulse0.width"),
        world_api.state.get_float("lightning.pulse0.strength")
    );
    let pulse1 = lightning_pulse(
        phase,
        world_api.state.get_float("lightning.pulse1.center"),
        world_api.state.get_float("lightning.pulse1.width"),
        world_api.state.get_float("lightning.pulse1.strength")
    );
    let pulse2 = lightning_pulse(
        phase,
        world_api.state.get_float("lightning.pulse2.center"),
        world_api.state.get_float("lightning.pulse2.width"),
        world_api.state.get_float("lightning.pulse2.strength")
    );

    let flash = maxf(pulse0, maxf(pulse1, pulse2));
    if world_api.state.get_bool("lightning.major") {
        let hold = 1.0 - absf(phase - 0.50) / 0.42;
        if hold > 0.0 {
            return maxf(flash, hold * 1.75);
        }
    }

    flash
}

fn update_rain(world_api, time, far_light, mid_light, near_light) {
    if time >= world_api.state.get_float("rain.next_gust") {
        let duration = world_api.random.range(0.8, 2.4);
        let strength = world_api.random.range(0.35, 1.0);
        let direction = if world_api.random.range(0.0, 1.0) < 0.18 {
            -1.0
        } else {
            1.0
        };

        world_api.state.set_float("rain.gust_start", time);
        world_api.state.set_float("rain.gust_until", time + duration);
        world_api.state.set_float("rain.gust_duration", duration);
        world_api.state.set_float("rain.gust_strength", strength);
        world_api.state.set_float("rain.gust_dir", direction);
        world_api.state.set_float("rain.next_gust", time + duration + world_api.random.range(4.0, 10.0));
    }

    let gust = if time < world_api.state.get_float("rain.gust_until") {
        let phase = clamp01((time - world_api.state.get_float("rain.gust_start")) / world_api.state.get_float("rain.gust_duration"));
        let envelope = (phase * 3.14159).sin();
        envelope * world_api.state.get_float("rain.gust_strength")
    } else {
        0.0
    };
    let dir = world_api.state.get_float("rain.gust_dir");

    world_api.particles.set_intensity(rain_far_entity(), clamp01(far_light * 0.62 + gust * 0.16));
    world_api.particles.set_intensity(rain_mid_entity(), clamp01(mid_light * 0.52 + gust * 0.14));
    world_api.particles.set_intensity(rain_near_entity(), clamp01(near_light * 0.42 + gust * 0.12));
    world_api.particles.set_wind(
        rain_far_entity(),
        68.0 + dir * gust * 260.0,
        -320.0 - gust * 54.0,
        0.12 + gust * 0.22
    );
    world_api.particles.set_wind(
        rain_mid_entity(),
        112.0 + dir * gust * 430.0,
        -500.0 - gust * 92.0,
        0.16 + gust * 0.27
    );
    world_api.particles.set_wind(
        rain_near_entity(),
        150.0 + dir * gust * 620.0,
        -680.0 - gust * 130.0,
        0.18 + gust * 0.32
    );
}

fn on_enter() {
    let background = background_entity();
    let lightning = world.light2d.get_light(lightning_id());

    world.state.reset_scene();
    seed_menu_state(world);
    reset_intro_state(world);
    reset_rain_state(world);
    world.entities.hide(menu_entity());
    world.entities.hide(press_any_key_entity());
    world.layered_image2d.set_base_opacity(background, 0.0);
    lightning.set_color("#B8D8FFFF");
    lightning.set_intensity(0.0);
    set_intro_layer_opacities(world, background, 0.0);
    world.dev.log("They Are Rotten main menu loaded.");
}

fn update(dt) {
    let background = background_entity();
    let lightning = world.light2d.get_light(lightning_id());
    let time = world.state.get_float("intro.t") + dt;
    world.state.set_float("intro.t", time);

    let intro_done = time >= 10.0;
    update_neon_flicker(world, time, intro_done);
    update_pharmacy_fault(world, time, intro_done);

    let flash = lightning_flash(world, time);
    let neon_mul = world.state.get_float("neon.mul");
    let pharmacy_mul = world.state.get_float("pharmacy.mul");
    lightning.set_intensity(flash);

    let background_reveal = fade(time, 0.0, 4.0);
    let bar_reveal = fade(time, 4.6, 1.0);
    let neon_reveal = fade(time, 6.2, 1.8);
    let pharmacy_reveal = fade(time, 8.4, 1.6);

    let base = clamp01(flash * 0.88);
    let skyline = clamp01(background_reveal + flash * 0.92);
    let skyline_blur = clamp01(skyline * wave(time, 0.38, 0.4, 0.62, 1.05) + flash * 0.85);
    let beacons = clamp01(background_reveal * 0.45 + flash * 0.46);
    let beacons_blur = clamp01(beacons * wave(time, 0.72, 1.8, 0.30, 0.58) + flash * 0.36);
    let bar = clamp01(bar_reveal + flash * 0.32);
    let bar_blur = clamp01(bar * wave(time, 0.30, 2.6, 0.08, 0.18) + flash * 0.08);
    let lanterns = clamp01(bar_reveal * 0.85 + flash * 0.26);
    let lanterns_blur = clamp01(lanterns * wave(time, 0.34, 3.4, 0.10, 0.22) + flash * 0.08);
    let club_false_start = fade(time, 5.45, 0.16) * (1.0 - fade(time, 5.70, 0.16)) * 0.32;
    let club = clamp01((neon_reveal * 0.62 + club_false_start) * neon_mul + flash * 0.26);
    let club_blur = clamp01(club * wave(time, 0.64, 0.9, 0.28, 0.58) + flash * 0.20);
    let entry = clamp01(neon_reveal * 0.40 * neon_mul + flash * 0.20);
    let entry_blur = clamp01(entry * wave(time, 0.58, 2.0, 0.20, 0.44) + flash * 0.14);
    let pharmacy = clamp01(pharmacy_reveal * 0.68 * pharmacy_mul + flash * 0.28);
    let pharmacy_blur = clamp01(pharmacy * wave(time, 0.52, 4.1, 0.22, 0.52) + flash * 0.18);
    update_rain(world, time, skyline, maxf(club, entry), maxf(bar, lanterns));

    world.layered_image2d.set_base_opacity(background, base);
    world.layered_image2d.set_opacity(background, "skyline", skyline);
    world.layered_image2d.set_opacity(background, "skyline_blur", skyline_blur);
    world.layered_image2d.set_opacity(background, "aircraft_beacons", beacons);
    world.layered_image2d.set_opacity(background, "aircraft_beacons_blur", beacons_blur);
    world.layered_image2d.set_opacity(background, "club_sign", club);
    world.layered_image2d.set_opacity(background, "club_sign_blur", club_blur);
    world.layered_image2d.set_opacity(background, "club_entry", entry);
    world.layered_image2d.set_opacity(background, "club_entry_blur", entry_blur);
    world.layered_image2d.set_opacity(background, "bar_sign", bar);
    world.layered_image2d.set_opacity(background, "bar_sign_blur", bar_blur);
    world.layered_image2d.set_opacity(background, "bar_lanterns", lanterns);
    world.layered_image2d.set_opacity(background, "bar_lanterns_blur", lanterns_blur);
    world.layered_image2d.set_opacity(background, "pharmacy_cross", pharmacy);
    world.layered_image2d.set_opacity(background, "pharmacy_cross_blur", pharmacy_blur);

    if intro_done && !world.state.get_bool("intro.prompt_shown") {
        world.entities.show(press_any_key_entity());
        world.state.set_bool("intro.prompt_shown", true);
    }

    if world.state.get_bool("intro.prompt_shown") && !world.state.get_bool("intro.menu_active") {
        if world.actions.pressed("menu.any") {
            world.entities.hide(press_any_key_entity());
            world.entities.show(menu_entity());
            world.state.set_bool("intro.menu_active", true);
        }
    }
}

fn on_event(topic, payload) {
    if topic == "they-are-rotten.menu.start" {
        world.dev.log("Main Menu: START selected.");
        return;
    }

    if topic == "they-are-rotten.menu.options" {
        world.dev.log("Main Menu: OPTIONS selected.");
        return;
    }

    if topic == "they-are-rotten.menu.quit" {
        world.dev.log("Main Menu: QUIT selected.");
        return;
    }
}
