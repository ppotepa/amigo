//! glTF keyframes: clamped sampler ranges, shortest-path quaternion LINEAR,
//! and time-scaled Hermite CUBICSPLINE (normalized after quaternion sampling).
use glam::Quat;
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MeshAnimationProperty {
    Translation,
    Rotation,
    Scale,
    Weights,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MeshAnimationInterpolation {
    Step,
    Linear,
    CubicSpline,
}

#[derive(Debug, Clone, Serialize)]
pub struct MeshAnimationTrack {
    pub node: usize,
    pub node_path: String,
    pub property: MeshAnimationProperty,
    pub interpolation: MeshAnimationInterpolation,
    pub keyframes: usize,
    #[serde(skip)]
    pub(super) times: Arc<[f32]>,
    #[serde(skip)]
    pub(super) values: Arc<[f32]>,
    #[serde(skip)]
    pub(super) dimension: usize,
}
#[derive(Debug, Clone, Serialize)]
pub struct MeshAnimationClip {
    /// File-array identity; display names are not required to be unique.
    pub index: usize,
    pub name: String,
    pub duration_seconds: f32,
    pub tracks: Vec<MeshAnimationTrack>,
}
impl MeshAnimationTrack {
    pub(super) fn validate(&self) -> Result<(), String> {
        if self.times.is_empty()
            || self.dimension == 0
            || self.times.iter().any(|t| !t.is_finite() || *t < 0.0)
            || self.times.windows(2).any(|t| t[0] >= t[1])
            || self.values.iter().any(|v| !v.is_finite())
        {
            return Err("invalid animation timestamps or values".into());
        }
        let factor = if self.interpolation == MeshAnimationInterpolation::CubicSpline {
            3
        } else {
            1
        };
        if self
            .times
            .len()
            .checked_mul(self.dimension)
            .and_then(|n| n.checked_mul(factor))
            != Some(self.values.len())
            || (factor == 3 && self.times.len() < 2)
        {
            return Err("animation sampler input/output counts disagree".into());
        }
        if self.property == MeshAnimationProperty::Rotation {
            for key in 0..self.times.len() {
                let value = self.key_value(key);
                let norm = value.iter().map(|x| x * x).sum::<f32>();
                if !norm.is_finite() || norm <= 1e-12 {
                    return Err("invalid animation quaternion".into());
                }
            }
        }
        Ok(())
    }
    fn key_value(&self, key: usize) -> &[f32] {
        let start = if self.interpolation == MeshAnimationInterpolation::CubicSpline {
            key * 3 + 1
        } else {
            key
        } * self.dimension;
        &self.values[start..start + self.dimension]
    }
    pub(super) fn sample(&self, time: f32) -> Result<Vec<f32>, String> {
        let upper = self.times.partition_point(|t| *t <= time);
        let mut value = if upper == 0 {
            self.key_value(0).to_vec()
        } else if upper == self.times.len() {
            self.key_value(upper - 1).to_vec()
        } else {
            let lower = upper - 1;
            let duration = self.times[upper] - self.times[lower];
            let t = (time - self.times[lower]) / duration;
            let a = self.key_value(lower);
            let b = self.key_value(upper);
            match self.interpolation {
                MeshAnimationInterpolation::Step => a.to_vec(),
                MeshAnimationInterpolation::Linear
                    if self.property == MeshAnimationProperty::Rotation =>
                {
                    let quat = |v: &[f32]| Quat::from_xyzw(v[0], v[1], v[2], v[3]).normalize();
                    quat(a).slerp(quat(b), t).to_array().to_vec()
                }
                MeshAnimationInterpolation::Linear => {
                    a.iter().zip(b).map(|(a, b)| a + (b - a) * t).collect()
                }
                MeshAnimationInterpolation::CubicSpline => {
                    let t2 = t * t;
                    let t3 = t2 * t;
                    let outgoing = (lower * 3 + 2) * self.dimension;
                    let incoming = upper * 3 * self.dimension;
                    (0..self.dimension)
                        .map(|i| {
                            (2.0 * t3 - 3.0 * t2 + 1.0) * a[i]
                                + duration * (t3 - 2.0 * t2 + t) * self.values[outgoing + i]
                                + (-2.0 * t3 + 3.0 * t2) * b[i]
                                + duration * (t3 - t2) * self.values[incoming + i]
                        })
                        .collect()
                }
            }
        };
        if self.property == MeshAnimationProperty::Rotation {
            let length = value.iter().map(|x| x * x).sum::<f32>().sqrt();
            if !length.is_finite() || length <= 1e-6 {
                return Err("animation produced a zero quaternion".into());
            }
            for component in &mut value {
                *component /= length;
            }
        }
        if value.iter().any(|v| !v.is_finite()) {
            return Err("animation produced non-finite values".into());
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn track(
        interpolation: MeshAnimationInterpolation,
        property: MeshAnimationProperty,
        dimension: usize,
        values: Vec<f32>,
    ) -> MeshAnimationTrack {
        MeshAnimationTrack {
            node: 0,
            node_path: "Root".into(),
            property,
            interpolation,
            keyframes: 2,
            times: vec![1.0, 3.0].into(),
            values: values.into(),
            dimension,
        }
    }
    #[test]
    fn step_and_linear_clamp_individual_sampler_ranges() {
        let mut t = track(
            MeshAnimationInterpolation::Step,
            MeshAnimationProperty::Weights,
            1,
            vec![2.0, 6.0],
        );
        t.validate().unwrap();
        assert_eq!(t.sample(0.0).unwrap(), [2.0]);
        assert_eq!(t.sample(2.0).unwrap(), [2.0]);
        assert_eq!(t.sample(3.0).unwrap(), [6.0]);
        t.interpolation = MeshAnimationInterpolation::Linear;
        assert_eq!(t.sample(2.0).unwrap(), [4.0]);
        assert_eq!(t.sample(4.0).unwrap(), [6.0]);
    }
    #[test]
    fn cubic_tangents_are_scaled_by_interval_duration() {
        let t = track(
            MeshAnimationInterpolation::CubicSpline,
            MeshAnimationProperty::Weights,
            1,
            vec![0.0, 0.0, 2.0, 0.0, 0.0, 0.0],
        );
        t.validate().unwrap();
        assert!((t.sample(2.0).unwrap()[0] - 0.5).abs() < 1e-6);
    }
    #[test]
    fn quaternion_linear_uses_the_short_arc_and_cubic_normalizes() {
        let t = track(
            MeshAnimationInterpolation::Linear,
            MeshAnimationProperty::Rotation,
            4,
            vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0],
        );
        t.validate().unwrap();
        assert!((t.sample(2.0).unwrap()[3].abs() - 1.0).abs() < 1e-6);
        let t = track(
            MeshAnimationInterpolation::CubicSpline,
            MeshAnimationProperty::Rotation,
            4,
            vec![
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            ],
        );
        t.validate().unwrap();
        assert!((t.sample(2.0).unwrap().iter().map(|x| x * x).sum::<f32>() - 1.0).abs() < 1e-6);
    }
}
