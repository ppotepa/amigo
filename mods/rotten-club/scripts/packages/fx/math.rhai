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


