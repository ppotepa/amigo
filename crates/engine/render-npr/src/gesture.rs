//! Deterministic hand-gesture shaping in screen space.
use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GestureSample {
    pub offset: f32,
    pub pressure: f32,
    pub correction: f32,
    pub grain: f32,
    /// Low-frequency material continuity along a gesture. This is distinct
    /// from geometric offset and screen-space paper tooth.
    pub deposit: f32,
}

fn hash(seed: u64, id: u32, lane: u64) -> f32 {
    let mut value = seed
        .wrapping_add(u64::from(id).wrapping_mul(0x9e37_79b9))
        .wrapping_add(lane.wrapping_mul(0x517c_c1b7));
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    (value as u32 as f32) / u32::MAX as f32
}

pub fn sample(
    seed: u64,
    id: u32,
    t: f32,
    confidence: f32,
    correction: f32,
    wobble: f32,
    variant: u32,
) -> GestureSample {
    let t = t.clamp(0.0, 1.0);
    let confidence = confidence.clamp(0.0, 1.0);
    let correction = correction.clamp(0.0, 1.0);
    let endpoint = (std::f32::consts::PI * t).sin().max(0.0);
    let phase = hash(seed, id, u64::from(variant) * 7 + 1) * std::f32::consts::TAU;
    let broad = (t * std::f32::consts::TAU * 1.13 + phase).sin() * 0.58
        + (t * std::f32::consts::TAU * 2.41 + phase * 0.71).sin() * 0.27;
    let correction_wave = (t * std::f32::consts::TAU * 6.7 + phase * 1.37).sin()
        * (0.55 + hash(seed, id, u64::from(variant) * 7 + 2) * 0.45);
    let jitter =
        wobble * endpoint * ((1.0 - confidence) * broad + correction * correction_wave * 0.32);
    let pressure_wave = (t * std::f32::consts::TAU * 1.71 + phase * 0.42).sin() * 0.10
        + (t * std::f32::consts::TAU * 3.2 + phase).sin() * 0.045;
    let pressure = (0.62 + confidence * 0.28 + pressure_wave * (1.0 - confidence)).clamp(0.18, 1.0)
        * (0.76 + endpoint * 0.24);
    let grain = hash(
        seed ^ 0xa5a5_5a5a_1234_5678,
        id,
        (t * 4096.0) as u64 + u64::from(variant),
    );
    let deposit_phase =
        hash(seed ^ 0x38d4_92ab_76c1_0fe5, id, u64::from(variant) + 17) * std::f32::consts::TAU;
    let continuity = 0.5
        + 0.5
            * (t * std::f32::consts::TAU * 2.73
                + deposit_phase
                + (t * std::f32::consts::TAU * 5.17 + deposit_phase * 0.63).sin() * 0.41)
                .sin();
    let fleck = hash(
        seed ^ 0xd16b_54a3_b8e2_0c79,
        id,
        (t * 23.0).floor() as u64 + u64::from(variant) * 29,
    );
    // Sparse cells are allowed to lose almost all deposit. The caller decides
    // whether the chosen physical tool exposes this variation.
    let gap = ((fleck - 0.79) / 0.21).clamp(0.0, 1.0);
    let deposit = (0.68 + continuity * 0.32 - gap * 0.78).clamp(0.0, 1.0);
    GestureSample {
        offset: jitter,
        pressure,
        correction: correction_wave * endpoint,
        grain,
        deposit,
    }
}

/// Ramer-Douglas-Peucker simplification that keeps the original depth at the
/// surviving points. It is intentionally deterministic and never removes ends.
pub fn simplify(points: &[(Vec2, f32)], tolerance: f32) -> Vec<(Vec2, f32)> {
    if points.len() < 3 || tolerance <= 0.0 {
        return points.to_vec();
    }
    let tolerance_squared = tolerance * tolerance;
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    simplify_range(points, 0, points.len() - 1, tolerance_squared, &mut keep);
    points
        .iter()
        .zip(keep)
        .filter_map(|(point, keep)| keep.then_some(*point))
        .collect()
}

fn simplify_range(
    points: &[(Vec2, f32)],
    start: usize,
    end: usize,
    tolerance_squared: f32,
    keep: &mut [bool],
) {
    if end <= start + 1 {
        return;
    }
    let a = points[start].0;
    let b = points[end].0;
    let direction = b - a;
    let length_squared = direction.length_squared();
    let mut farthest = 0.0;
    let mut index = None;
    for (offset, point) in points[start + 1..end].iter().enumerate() {
        let distance = if length_squared < 1e-8 {
            point.0.distance_squared(a)
        } else {
            let t = ((point.0 - a).dot(direction) / length_squared).clamp(0.0, 1.0);
            point.0.distance_squared(a + direction * t)
        };
        if distance > farthest {
            farthest = distance;
            index = Some(start + 1 + offset);
        }
    }
    if farthest > tolerance_squared {
        let index = index.unwrap();
        keep[index] = true;
        simplify_range(points, start, index, tolerance_squared, keep);
        simplify_range(points, index, end, tolerance_squared, keep);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposit_is_seeded_bounded_and_varies_along_a_gesture() {
        let samples = (0..32)
            .map(|index| sample(17, 91, index as f32 / 31.0, 0.4, 0.6, 1.0, 0))
            .collect::<Vec<_>>();
        assert_eq!(
            samples,
            (0..32)
                .map(|index| sample(17, 91, index as f32 / 31.0, 0.4, 0.6, 1.0, 0))
                .collect::<Vec<_>>()
        );
        assert!(
            samples
                .iter()
                .all(|sample| (0.0..=1.0).contains(&sample.deposit))
        );
        assert!(
            samples
                .windows(2)
                .any(|pair| (pair[0].deposit - pair[1].deposit).abs() > 0.01)
        );
    }
}
