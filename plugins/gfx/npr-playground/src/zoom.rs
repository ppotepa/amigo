//! Transient viewport motion; never serialized into scene presets.
#[derive(Default)]
pub(crate) struct SmoothZoom {
    target_log: f64,
    last_distance: Option<f32>,
}
impl SmoothZoom {
    pub fn advance(&mut self, distance: f32, wheel: f32, seconds: f32) -> f32 {
        debug_assert!(distance.is_finite(), "camera settings must be validated");
        let distance = distance.clamp(0.1, 100.0);
        // A panel edit, camera fit, preset or undo takes precedence over pending motion.
        if self.last_distance != Some(distance) {
            self.target_log = f64::from(distance).ln();
        }
        if wheel.is_finite() {
            self.target_log =
                (self.target_log - f64::from(wheel) * 0.1).clamp(0.1_f64.ln(), 100.0_f64.ln());
        }
        let current = f64::from(distance).ln();
        let alpha = if seconds.is_finite() && seconds > 0.0 {
            -(-18.0 * f64::from(seconds)).exp_m1()
        } else {
            0.0
        };
        let next = current + (self.target_log - current) * alpha;
        let next = if (self.target_log - next).abs() < 0.0001 {
            self.target_log
        } else {
            next
        };
        let distance = (next.exp() as f32).clamp(0.1, 100.0);
        self.last_distance = Some(distance);
        distance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zoom_is_smooth_monotonic_and_reversible() {
        let mut zoom = SmoothZoom::default();
        let target = 10.0 * (-0.1_f32).exp();
        let mut distance = zoom.advance(10.0, 1.0, 1.0 / 60.0);
        assert!(distance > target && distance < 10.0);
        for _ in 0..120 {
            let next = zoom.advance(distance, 0.0, 1.0 / 60.0);
            assert!(next <= distance && next >= target - 0.00001);
            distance = next;
        }
        assert!((distance - target).abs() < 0.00001);
        distance = zoom.advance(distance, -1.0, 1.0 / 60.0);
        for _ in 0..120 {
            distance = zoom.advance(distance, 0.0, 1.0 / 60.0);
        }
        assert!((distance - 10.0).abs() < 0.0001);
    }
    #[test]
    fn response_is_independent_of_frame_rate() {
        let sample = |fps: u32| {
            let mut zoom = SmoothZoom::default();
            let mut d = zoom.advance(14.0, 3.0, 0.0);
            for _ in 0..fps / 10 {
                d = zoom.advance(d, 0.0, 1.0 / fps as f32);
            }
            d
        };
        for fps in [20, 30, 60, 120] {
            assert!((sample(fps) - sample(240)).abs() < 0.0001);
        }
    }
    #[test]
    fn external_edits_limits_and_fractional_wheel_are_respected() {
        let mut zoom = SmoothZoom::default();
        zoom.advance(10.0, 4.0, 0.01);
        assert_eq!(zoom.advance(20.0, 0.0, 0.1), 20.0);
        assert_eq!(zoom.advance(20.0, f32::NAN, f32::NAN), 20.0);
        assert_eq!(zoom.advance(20.0, f32::MAX, 1.0), 0.1);
        assert_eq!(zoom.advance(0.1, -f32::MAX, 1.0), 100.0);
        let mut split = SmoothZoom::default();
        let mut d = 10.0;
        for _ in 0..10 {
            d = split.advance(d, 0.1, 0.0);
        }
        d = split.advance(d, 0.0, 1.0);
        assert!((d - SmoothZoom::default().advance(10.0, 1.0, 1.0)).abs() < 0.00001);
    }
}
