mod tests {
    use super::{Curve1d, CurvePoint1d};

    #[test]
    fn linear_returns_t() {
        assert_eq!(Curve1d::Linear.sample(0.25), 0.25);
    }

    #[test]
    fn constant_ignores_t() {
        assert_eq!(Curve1d::Constant(0.7).sample(0.25), 0.7);
    }

    #[test]
    fn ease_in_starts_slow() {
        assert!(Curve1d::EaseIn.sample(0.5) < Curve1d::Linear.sample(0.5));
    }

    #[test]
    fn ease_out_starts_fast() {
        assert!(Curve1d::EaseOut.sample(0.5) > Curve1d::Linear.sample(0.5));
    }

    #[test]
    fn smoothstep_has_zero_and_one_endpoints() {
        assert_eq!(Curve1d::SmoothStep.sample(0.0), 0.0);
        assert_eq!(Curve1d::SmoothStep.sample(1.0), 1.0);
    }

    #[test]
    fn custom_interpolates_between_points() {
        let curve = Curve1d::Custom {
            points: vec![
                CurvePoint1d { t: 0.0, value: 2.0 },
                CurvePoint1d {
                    t: 1.0,
                    value: 10.0,
                },
            ],
        };
        assert_eq!(curve.sample(0.5), 6.0);
    }

    #[test]
    fn custom_clamps_before_first_point() {
        let curve = Curve1d::Custom {
            points: vec![
                CurvePoint1d {
                    t: 0.25,
                    value: 3.0,
                },
                CurvePoint1d { t: 1.0, value: 6.0 },
            ],
        };
        assert_eq!(curve.sample(0.0), 3.0);
    }

    #[test]
    fn custom_clamps_after_last_point() {
        let curve = Curve1d::Custom {
            points: vec![
                CurvePoint1d { t: 0.0, value: 3.0 },
                CurvePoint1d {
                    t: 0.75,
                    value: 6.0,
                },
            ],
        };
        assert_eq!(curve.sample(1.0), 6.0);
    }

    #[test]
    fn custom_handles_unsorted_points_defensively() {
        let curve = Curve1d::Custom {
            points: vec![
                CurvePoint1d {
                    t: 1.0,
                    value: 10.0,
                },
                CurvePoint1d { t: 0.0, value: 2.0 },
            ],
        };
        assert_eq!(curve.sample(0.5), 6.0);
    }

    #[test]
    fn custom_handles_empty_points() {
        assert_eq!(Curve1d::Custom { points: vec![] }.sample(0.5), 1.0);
    }
}
