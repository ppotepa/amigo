import "pkg:amigo.std/input" as input;

let square = world.entities.named("playground-2d-ui-preview-square");

let hp = 0.82;
let energy = 0.55;
let threat = 0.35;
let shield_online = true;
let repair_cooldown = 0.0;
let mission_time = 0.0;
let toast_time = 0.0;
let toast_message = "";

fn subtitle_path() { "playground-2d-ui-preview.root.hud-card.subtitle" }
fn hp_bar_path() { "playground-2d-ui-preview.root.hud-card.hp-bar" }
fn energy_bar_path() { "playground-2d-ui-preview.root.hud-card.energy-bar" }
fn threat_bar_path() { "playground-2d-ui-preview.root.hud-card.threat-bar" }
fn shield_value_path() { "playground-2d-ui-preview.root.hud-card.status-row.shield-box.shield-value" }
fn system_value_path() { "playground-2d-ui-preview.root.hud-card.status-row.system-box.system-value" }
fn objective_value_path() { "playground-2d-ui-preview.root.mission-card.objective-value" }
fn phase_value_path() { "playground-2d-ui-preview.root.mission-card.phase-value" }
fn repair_button_path() { "playground-2d-ui-preview.root.control-card.button-row.repair-button" }
fn shield_button_path() { "playground-2d-ui-preview.root.control-card.button-row.shield-button" }
fn stabilize_button_path() { "playground-2d-ui-preview.root.control-card.stabilize-button" }
fn event_toast_path() { "playground-2d-ui-preview.root.event-toast" }
fn event_toast_text_path() { "playground-2d-ui-preview.root.event-toast.event-toast-text" }
fn critical_alert_path() { "playground-2d-ui-preview.root.critical-alert" }

fn clamp01(value) {
    if value < 0.0 { return 0.0; }
    if value > 1.0 { return 1.0; }
    value
}

fn phase_label(current_time) {
    if current_time < 10.0 { return "WAVE 01"; }
    if current_time < 20.0 { return "WAVE 02"; }
    "WAVE 03"
}

fn objective_label(current_hp, shield_active, current_threat) {
    if current_hp < 0.18 { return "OBJECTIVE: STABILIZE HULL"; }
    if !shield_active { return "OBJECTIVE: RESTORE SHIELD"; }
    if current_threat > 0.72 { return "OBJECTIVE: BLEED THREAT"; }
    "OBJECTIVE: SURVIVE CALIBRATION"
}

fn status_label(current_hp, current_threat) {
    if current_hp < 0.18 { return "CRITICAL"; }
    if current_hp < 0.45 || current_threat > 0.75 { return "WARNING"; }
    "STABLE"
}

fn subtitle_label(current_hp, shield_active, current_threat) {
    if current_hp < 0.18 { return "Critical damage detected"; }
    if !shield_active { return "Shield offline - hull exposed"; }
    if current_threat > 0.75 { return "Threat spike detected"; }
    "Rhai combat loop active"
}

fn on_enter() {
    world.dev.event("playground-2d.ui-preview.entered", world.scene.current_id());
    world.ui.hide(event_toast_path());
    world.ui.hide(critical_alert_path());
    world.ui.set_text(subtitle_path(), subtitle_label(hp, shield_online, threat));
    world.ui.set_text(shield_value_path(), if shield_online { "ONLINE" } else { "OFFLINE" });
    world.ui.set_text(system_value_path(), status_label(hp, threat));
    world.ui.set_text(objective_value_path(), objective_label(hp, shield_online, threat));
    world.ui.set_text(phase_value_path(), phase_label(mission_time));
    world.ui.set_text(shield_button_path(), "Disable Shield");
    world.ui.enable(repair_button_path());
    world.ui.set_text(repair_button_path(), "Repair Pulse");
    world.ui.enable(stabilize_button_path());
    world.ui.set_value(hp_bar_path(), hp);
    world.ui.set_value(energy_bar_path(), energy);
    world.ui.set_value(threat_bar_path(), threat);
}

fn update(dt) {
    mission_time = mission_time + dt;
    square.rotate_2d(dt * (0.45 + threat));

    threat = clamp01(threat + dt * 0.04);
    energy = clamp01(energy + dt * 0.06);

    if shield_online {
        energy = clamp01(energy - dt * 0.02);
    }

    if !shield_online && threat > 0.6 {
        hp = clamp01(hp - dt * 0.1);
    }

    if threat > 0.85 {
        hp = clamp01(hp - dt * 0.05);
    }

    let move_x = input::axis(world, "preview.move_x");
    let move_y = input::axis(world, "preview.move_y");

    if move_x < -0.01 {
        hp = clamp01(hp - dt * 0.3);
    }

    if move_x > 0.01 {
        hp = clamp01(hp + dt * 0.2);
    }

    if move_y > 0.01 {
        energy = clamp01(energy + dt * 0.3);
    }

    if move_y < -0.01 {
        threat = clamp01(threat - dt * 0.3);
    }

    if input::pressed(world, "preview.repair") {
        if repair_cooldown > 0.0 {
            toast_message = "REPAIR COOLING";
            toast_time = 0.9;
        } else if energy < 0.2 {
            toast_message = "NOT ENOUGH ENERGY";
            toast_time = 0.9;
        } else {
            hp = clamp01(hp + 0.24);
            energy = clamp01(energy - 0.2);
            threat = clamp01(threat - 0.08);
            repair_cooldown = 2.4;
            toast_message = "REPAIR PULSE ACTIVATED";
            toast_time = 1.2;
        }
    }

    if input::pressed(world, "preview.toggle_shield") {
        shield_online = !shield_online;
        if shield_online {
            toast_message = "SHIELD ONLINE";
            toast_time = 1.0;
        } else {
            toast_message = "SHIELD OFFLINE";
            toast_time = 1.0;
        }
    }

    if input::pressed(world, "preview.stabilize") {
        if energy < 0.15 {
            toast_message = "CAPACITOR TOO LOW";
            toast_time = 0.9;
        } else {
            energy = clamp01(energy - 0.15);
            threat = clamp01(threat - 0.26);
            toast_message = "THREAT DAMPENED";
            toast_time = 1.0;
        }
    }

    let had_cooldown = repair_cooldown > 0.0;
    if repair_cooldown > 0.0 {
        repair_cooldown = repair_cooldown - dt;
        if repair_cooldown < 0.0 {
            repair_cooldown = 0.0;
        }
    }

    if had_cooldown && repair_cooldown == 0.0 {
        toast_message = "REPAIR READY";
        toast_time = 0.8;
    }

    if toast_time > 0.0 {
        toast_time = toast_time - dt;
        if toast_time < 0.0 {
            toast_time = 0.0;
            toast_message = "";
        }
    }

    world.ui.set_text(subtitle_path(), subtitle_label(hp, shield_online, threat));
    world.ui.set_text(shield_value_path(), if shield_online { "ONLINE" } else { "OFFLINE" });
    world.ui.set_text(system_value_path(), status_label(hp, threat));
    world.ui.set_text(objective_value_path(), objective_label(hp, shield_online, threat));
    world.ui.set_text(phase_value_path(), phase_label(mission_time));

    if shield_online {
        world.ui.set_text(shield_button_path(), "Disable Shield");
    } else {
        world.ui.set_text(shield_button_path(), "Enable Shield");
    }

    if repair_cooldown > 0.0 {
        world.ui.disable(repair_button_path());
        world.ui.set_text(repair_button_path(), "Repair Cooling");
    } else {
        world.ui.enable(repair_button_path());
        world.ui.set_text(repair_button_path(), "Repair Pulse");
    }

    if energy < 0.15 {
        world.ui.disable(stabilize_button_path());
    } else {
        world.ui.enable(stabilize_button_path());
    }

    if hp < 0.18 {
        world.ui.show(critical_alert_path());
    } else {
        world.ui.hide(critical_alert_path());
    }

    if toast_time > 0.0 {
        world.ui.set_text(event_toast_text_path(), toast_message);
        world.ui.show(event_toast_path());
    } else {
        world.ui.hide(event_toast_path());
    }

    world.ui.set_value(hp_bar_path(), hp);
    world.ui.set_value(energy_bar_path(), energy);
    world.ui.set_value(threat_bar_path(), threat);
}

fn on_event(topic, payload) {
    if topic == "playground-2d.ui-preview.repair-clicked" {
        if repair_cooldown > 0.0 {
            toast_message = "REPAIR COOLING";
            toast_time = 0.9;
        } else if energy < 0.2 {
            toast_message = "NOT ENOUGH ENERGY";
            toast_time = 0.9;
        } else {
            hp = clamp01(hp + 0.24);
            energy = clamp01(energy - 0.2);
            threat = clamp01(threat - 0.08);
            repair_cooldown = 2.4;
            toast_message = "REPAIR PULSE ACTIVATED";
            toast_time = 1.2;
        }
    }

    if topic == "playground-2d.ui-preview.shield-clicked" {
        shield_online = !shield_online;
        if shield_online {
            toast_message = "SHIELD ONLINE";
            toast_time = 1.0;
        } else {
            toast_message = "SHIELD OFFLINE";
            toast_time = 1.0;
        }
    }

    if topic == "playground-2d.ui-preview.stabilize-clicked" {
        if energy < 0.15 {
            toast_message = "CAPACITOR TOO LOW";
            toast_time = 0.9;
        } else {
            energy = clamp01(energy - 0.15);
            threat = clamp01(threat - 0.26);
            toast_message = "THREAT DAMPENED";
            toast_time = 1.0;
        }
    }

    world.ui.set_text(subtitle_path(), subtitle_label(hp, shield_online, threat));
    world.ui.set_text(shield_value_path(), if shield_online { "ONLINE" } else { "OFFLINE" });
    world.ui.set_text(system_value_path(), status_label(hp, threat));
    world.ui.set_text(objective_value_path(), objective_label(hp, shield_online, threat));
    world.ui.set_text(phase_value_path(), phase_label(mission_time));

    if shield_online {
        world.ui.set_text(shield_button_path(), "Disable Shield");
    } else {
        world.ui.set_text(shield_button_path(), "Enable Shield");
    }

    if repair_cooldown > 0.0 {
        world.ui.disable(repair_button_path());
        world.ui.set_text(repair_button_path(), "Repair Cooling");
    } else {
        world.ui.enable(repair_button_path());
        world.ui.set_text(repair_button_path(), "Repair Pulse");
    }

    if energy < 0.15 {
        world.ui.disable(stabilize_button_path());
    } else {
        world.ui.enable(stabilize_button_path());
    }

    if hp < 0.18 {
        world.ui.show(critical_alert_path());
    } else {
        world.ui.hide(critical_alert_path());
    }

    if toast_time > 0.0 {
        world.ui.set_text(event_toast_text_path(), toast_message);
        world.ui.show(event_toast_path());
    } else {
        world.ui.hide(event_toast_path());
    }

    world.ui.set_value(hp_bar_path(), hp);
    world.ui.set_value(energy_bar_path(), energy);
    world.ui.set_value(threat_bar_path(), threat);
}
