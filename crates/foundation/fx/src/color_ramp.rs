use std::cmp::Ordering;

use amigo_math::ColorRgba;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorStop {
    pub t: f32,
    pub color: ColorRgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorInterpolation {
    LinearRgb,
    Step,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorRamp {
    pub stops: Vec<ColorStop>,
    pub interpolation: ColorInterpolation,
}

impl ColorRamp {
    pub fn constant(color: ColorRgba) -> Self {
        Self {
            stops: vec![ColorStop { t: 0.0, color }],
            interpolation: ColorInterpolation::LinearRgb,
        }
    }

    pub fn sample(&self, t: f32) -> ColorRgba {
        let ramp = self.normalized();
        if ramp.stops.is_empty() {
            return ColorRgba::WHITE;
        }
        if ramp.stops.len() == 1 {
            return ramp.stops[0].color;
        }

        let t = finite_unit(t);
        if t <= ramp.stops[0].t {
            return ramp.stops[0].color;
        }
        let last = ramp.stops[ramp.stops.len() - 1];
        if t >= last.t {
            return last.color;
        }

        for pair in ramp.stops.windows(2) {
            let left = pair[0];
            let right = pair[1];
            if t >= left.t && t <= right.t {
                return match ramp.interpolation {
                    ColorInterpolation::LinearRgb => {
                        let span = (right.t - left.t).max(f32::EPSILON);
                        lerp_color(left.color, right.color, (t - left.t) / span)
                    }
                    ColorInterpolation::Step => left.color,
                };
            }
        }

        last.color
    }

    pub fn normalized(&self) -> Self {
        let mut stops = self
            .stops
            .iter()
            .copied()
            .filter(|stop| {
                stop.t.is_finite()
                    && stop.color.r.is_finite()
                    && stop.color.g.is_finite()
                    && stop.color.b.is_finite()
                    && stop.color.a.is_finite()
            })
            .map(|stop| ColorStop {
                t: finite_unit(stop.t),
                color: ColorRgba::new(
                    stop.color.r.clamp(0.0, 1.0),
                    stop.color.g.clamp(0.0, 1.0),
                    stop.color.b.clamp(0.0, 1.0),
                    stop.color.a.clamp(0.0, 1.0),
                ),
            })
            .collect::<Vec<_>>();
        stops.sort_by(|left, right| left.t.partial_cmp(&right.t).unwrap_or(Ordering::Equal));
        Self {
            stops,
            interpolation: self.interpolation,
        }
    }
}

fn finite_unit(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn lerp_color(left: ColorRgba, right: ColorRgba, t: f32) -> ColorRgba {
    let t = finite_unit(t);
    ColorRgba::new(
        left.r + (right.r - left.r) * t,
        left.g + (right.g - left.g) * t,
        left.b + (right.b - left.b) * t,
        left.a + (right.a - left.a) * t,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red() -> ColorRgba {
        ColorRgba::new(1.0, 0.0, 0.0, 1.0)
    }

    fn blue() -> ColorRgba {
        ColorRgba::new(0.0, 0.0, 1.0, 0.5)
    }

    #[test]
    fn color_ramp_constant_returns_same_color() {
        let ramp = ColorRamp::constant(red());

        assert_eq!(ramp.sample(0.0), red());
        assert_eq!(ramp.sample(0.5), red());
        assert_eq!(ramp.sample(1.0), red());
    }

    #[test]
    fn color_ramp_linear_interpolates_rgb_and_alpha() {
        let ramp = ColorRamp {
            interpolation: ColorInterpolation::LinearRgb,
            stops: vec![
                ColorStop {
                    t: 0.0,
                    color: red(),
                },
                ColorStop {
                    t: 1.0,
                    color: blue(),
                },
            ],
        };

        let color = ramp.sample(0.5);

        assert!((color.r - 0.5).abs() < 0.001);
        assert_eq!(color.g, 0.0);
        assert!((color.b - 0.5).abs() < 0.001);
        assert!((color.a - 0.75).abs() < 0.001);
    }

    #[test]
    fn color_ramp_clamps_before_first_stop() {
        let ramp = ColorRamp {
            interpolation: ColorInterpolation::LinearRgb,
            stops: vec![ColorStop {
                t: 0.25,
                color: red(),
            }],
        };

        assert_eq!(ramp.sample(-1.0), red());
    }

    #[test]
    fn color_ramp_clamps_after_last_stop() {
        let ramp = ColorRamp {
            interpolation: ColorInterpolation::LinearRgb,
            stops: vec![
                ColorStop {
                    t: 0.0,
                    color: red(),
                },
                ColorStop {
                    t: 0.75,
                    color: blue(),
                },
            ],
        };

        assert_eq!(ramp.sample(2.0), blue());
    }

    #[test]
    fn color_ramp_sorts_unsorted_stops() {
        let ramp = ColorRamp {
            interpolation: ColorInterpolation::LinearRgb,
            stops: vec![
                ColorStop {
                    t: 1.0,
                    color: blue(),
                },
                ColorStop {
                    t: 0.0,
                    color: red(),
                },
            ],
        };

        assert_eq!(ramp.sample(0.0), red());
        assert_eq!(ramp.sample(1.0), blue());
    }

    #[test]
    fn color_ramp_step_interpolation_uses_previous_stop() {
        let ramp = ColorRamp {
            interpolation: ColorInterpolation::Step,
            stops: vec![
                ColorStop {
                    t: 0.0,
                    color: red(),
                },
                ColorStop {
                    t: 0.5,
                    color: blue(),
                },
            ],
        };

        assert_eq!(ramp.sample(0.25), red());
        assert_eq!(ramp.sample(0.75), blue());
    }
}
