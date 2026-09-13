pub(crate) fn deterministic_noise(seed: u64, edge: u64, pass: u64, salt: u64) -> f32 {
    let mut value = seed
        ^ edge.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ pass.wrapping_mul(0xBF58_476D_1CE4_E5B9)
        ^ salt.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;
    ((value >> 40) as f32) / ((1u64 << 24) as f32)
}

pub(crate) fn deterministic_signed_noise(seed: u64, edge: u64, pass: u64, salt: u64) -> f32 {
    deterministic_noise(seed, edge, pass, salt) * 2.0 - 1.0
}

pub(crate) fn coherent_signed_noise_1d(
    seed: u64,
    edge: u64,
    pass: u64,
    position: f32,
    salt: u64,
) -> f32 {
    let base = position.floor();
    let frac = (position - base).clamp(0.0, 1.0);
    let smooth = frac * frac * (3.0 - 2.0 * frac);
    let left = deterministic_signed_noise(seed, edge, pass, salt.wrapping_add(base as u64));
    let right = deterministic_signed_noise(
        seed,
        edge,
        pass,
        salt.wrapping_add(base as u64).wrapping_add(1),
    );
    left + (right - left) * smooth
}
