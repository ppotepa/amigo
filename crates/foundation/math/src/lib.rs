//! Low-level math primitives shared across 2D, 3D, UI, and FX domains.
//! It defines vectors, transforms, colors, and scalar curves without binding to any runtime system.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2 {
    pub translation: Vec2,
    pub rotation_radians: f32,
    pub scale: Vec2,
}

impl Default for Transform2 {
    fn default() -> Self {
        Self {
            translation: Vec2::ZERO,
            rotation_radians: 0.0,
            scale: Vec2::ONE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform3 {
    pub translation: Vec3,
    pub rotation_euler: Vec3,
    pub scale: Vec3,
}

impl Default for Transform3 {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation_euler: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorRgba {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurvePoint1d {
    pub t: f32,
    pub value: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Curve1d {
    Constant(f32),
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    SmoothStep,
    Custom {
        points: Vec<CurvePoint1d>,
    },
}

pub type ValueCurve = Curve1d;
pub type ValueCurvePoint = CurvePoint1d;

impl Curve1d {
    pub const fn constant(value: f32) -> Self {
        Self::Constant(value)
    }

    pub const fn linear() -> Self {
        Self::Linear
    }

    pub fn sample(&self, t: f32) -> f32 {
        self.sample_clamped(t)
    }

    pub fn sample_clamped(&self, t: f32) -> f32 {
        let t = if t.is_finite() {
            t.clamp(0.0, 1.0)
        } else {
            0.0
        };
        match self {
            Self::Constant(value) => *value,
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) * 0.5
                }
            }
            Self::SmoothStep => t * t * (3.0 - 2.0 * t),
            Self::Custom { points } => sample_custom_curve_1d(points, t),
        }
    }
}

fn sample_custom_curve_1d(points: &[CurvePoint1d], t: f32) -> f32 {
    let mut finite_points = points
        .iter()
        .copied()
        .filter(|point| point.t.is_finite() && point.value.is_finite())
        .collect::<Vec<_>>();
    if finite_points.is_empty() {
        return 1.0;
    }
    finite_points.sort_by(|left, right| {
        left.t
            .partial_cmp(&right.t)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if finite_points.len() == 1 {
        return finite_points[0].value;
    }
    if t <= finite_points[0].t {
        return finite_points[0].value;
    }
    if let Some(last) = finite_points.last()
        && t >= last.t
    {
        return last.value;
    }

    for window in finite_points.windows(2) {
        let left = window[0];
        let right = window[1];
        if t < left.t || t > right.t {
            continue;
        }
        let span = right.t - left.t;
        if span.abs() <= f32::EPSILON {
            return right.value;
        }
        let factor = ((t - left.t) / span).clamp(0.0, 1.0);
        return left.value + (right.value - left.value) * factor;
    }

    finite_points.last().map(|point| point.value).unwrap_or(1.0)
}

#[cfg(test)]
include!("tests.rs");
