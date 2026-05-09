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
    world_api.layered_image2d.set_opacity(background, "club_sign", value);
    world_api.layered_image2d.set_opacity(background, "club_entry", value);
    world_api.layered_image2d.set_opacity(background, "bar_sign", value);
    world_api.layered_image2d.set_opacity(background, "bar_lanterns", value);
    world_api.layered_image2d.set_opacity(background, "pharmacy_cross", value);
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
        let duration = world_api.random.range(0.10, 0.42);
        world_api.state.set_float("lightning.duration", duration);
        world_api.state.set_float("lightning.start", time);
        world_api.state.set_float("lightning.until", time + duration);
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
        world_api.state.set_float("lightning.next", time + duration + world_api.random.range(5.0, 15.0));
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

    maxf(pulse0, maxf(pulse1, pulse2))
}

fn on_enter() {
    let background = "they-are-rotten-main-menu-background";

    world.state.reset_scene();
    seed_menu_state(world);
    reset_intro_state(world);
    world.entities.hide("they-are-rotten-main-menu");
    world.entities.hide("they-are-rotten-main-menu-press-any-key");
    world.layered_image2d.set_base_opacity(background, 0.0);
    set_intro_layer_opacities(world, background, 0.0);
    world.dev.log("They Are Rotten main menu loaded.");
}

fn update(dt) {
    let background = "they-are-rotten-main-menu-background";
    let time = world.state.get_float("intro.t") + dt;
    world.state.set_float("intro.t", time);

    let intro_done = time >= 10.0;
    update_neon_flicker(world, time, intro_done);
    update_pharmacy_fault(world, time, intro_done);

    let flash = lightning_flash(world, time);
    let neon_mul = world.state.get_float("neon.mul");
    let pharmacy_mul = world.state.get_float("pharmacy.mul");

    let background_reveal = fade(time, 0.0, 4.0);
    let bar_reveal = fade(time, 4.6, 1.0);
    let neon_reveal = fade(time, 6.2, 1.8);
    let pharmacy_reveal = fade(time, 8.4, 1.6);

    let base = clamp01(flash * 0.88);
    let skyline = clamp01(background_reveal + flash * 0.92);
    let skyline_blur = clamp01(skyline * 0.55 + flash * 0.35);
    let beacons = clamp01(background_reveal * 0.45 + flash * 0.46);
    let bar = clamp01(bar_reveal + flash * 0.32);
    let lanterns = clamp01(bar_reveal * 0.85 + flash * 0.26);
    let club_false_start = fade(time, 5.45, 0.16) * (1.0 - fade(time, 5.70, 0.16)) * 0.32;
    let club = clamp01((neon_reveal * 0.62 + club_false_start) * neon_mul + flash * 0.26);
    let entry = clamp01(neon_reveal * 0.40 * neon_mul + flash * 0.20);
    let pharmacy = clamp01(pharmacy_reveal * 0.68 * pharmacy_mul + flash * 0.28);

    world.layered_image2d.set_base_opacity(background, base);
    world.layered_image2d.set_opacity(background, "skyline", skyline);
    world.layered_image2d.set_opacity(background, "skyline_blur", skyline_blur);
    world.layered_image2d.set_opacity(background, "aircraft_beacons", beacons);
    world.layered_image2d.set_opacity(background, "club_sign", club);
    world.layered_image2d.set_opacity(background, "club_entry", entry);
    world.layered_image2d.set_opacity(background, "bar_sign", bar);
    world.layered_image2d.set_opacity(background, "bar_lanterns", lanterns);
    world.layered_image2d.set_opacity(background, "pharmacy_cross", pharmacy);

    if intro_done && !world.state.get_bool("intro.prompt_shown") {
        world.entities.show("they-are-rotten-main-menu-press-any-key");
        world.state.set_bool("intro.prompt_shown", true);
    }

    if world.state.get_bool("intro.prompt_shown") && !world.state.get_bool("intro.menu_active") {
        if world.actions.pressed("menu.any") {
            world.entities.hide("they-are-rotten-main-menu-press-any-key");
            world.entities.show("they-are-rotten-main-menu");
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
