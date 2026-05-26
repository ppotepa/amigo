fn clamp(value, min_value, max_value) {
    if value < min_value {
        return min_value;
    }
    if value > max_value {
        return max_value;
    }
    value
}

fn clamp01(value) {
    clamp(value, 0.0, 1.0)
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

fn axis_delta(axis, step) {
    if axis > 0 {
        return step;
    }
    if axis < 0 {
        return -step;
    }
    0.0
}

fn ramp(time, start, duration) {
    smooth01((time - start) / duration)
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
