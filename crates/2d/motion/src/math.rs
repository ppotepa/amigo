use amigo_math::Curve1d;

pub(crate) fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else if target > current {
        current + max_delta
    } else {
        current - max_delta
    }
}

pub(crate) fn signed_curve_response(input: f32, curve: &Curve1d) -> f32 {
    if input.abs() <= f32::EPSILON {
        0.0
    } else {
        input.signum() * curve.sample(input.abs())
    }
}
