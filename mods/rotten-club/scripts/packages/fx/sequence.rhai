fn clamp01(value) {
    if value < 0.0 {
        return 0.0;
    }
    if value > 1.0 {
        return 1.0;
    }
    value
}

fn smooth01(value) {
    let t = clamp01(value);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a, b, t) {
    a + (b - a) * t
}

fn maxf(left, right) {
    if left > right {
        return left;
    }
    right
}

fn ramp(time, start, duration) {
    smooth01((time - start) / duration)
}

fn window(time, start, duration) {
    if time < start {
        return 0.0;
    }
    if time > start + duration {
        return 0.0;
    }
    clamp01((time - start) / duration)
}

fn pulse_long(time, start, rise, hold, decay) {
    if time < start {
        return 0.0;
    }

    let t = time - start;
    if t < rise {
        return smooth01(t / rise);
    }
    if t < rise + hold {
        return 1.0;
    }
    if t < rise + hold + decay {
        return 1.0 - smooth01((t - rise - hold) / decay);
    }
    0.0
}

fn once(world_api, key) {
    if world_api.state.get_bool(key) {
        return false;
    }

    world_api.state.set_bool(key, true);
    true
}
