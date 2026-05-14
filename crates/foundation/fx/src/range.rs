use amigo_math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalarRange {
    pub min: f32,
    pub max: f32,
}

impl ScalarRange {
    pub fn constant(value: f32) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    pub fn normalized(self) -> Self {
        if self.min <= self.max {
            self
        } else {
            Self {
                min: self.max,
                max: self.min,
            }
        }
    }

    pub fn sample(self, unit: f32) -> f32 {
        let range = self.normalized();
        let unit = if unit.is_finite() {
            unit.clamp(0.0, 1.0)
        } else {
            0.0
        };
        range.min + (range.max - range.min) * unit
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2Range {
    pub min: Vec2,
    pub max: Vec2,
}

impl Vec2Range {
    pub fn constant(value: Vec2) -> Self {
        Self {
            min: value,
            max: value,
        }
    }

    pub fn sample(self, x_unit: f32, y_unit: f32) -> Vec2 {
        Vec2::new(
            ScalarRange {
                min: self.min.x,
                max: self.max.x,
            }
            .sample(x_unit),
            ScalarRange {
                min: self.min.y,
                max: self.max.y,
            }
            .sample(y_unit),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_range_samples_between_min_and_max() {
        assert_eq!(ScalarRange { min: 2.0, max: 6.0 }.sample(0.5), 4.0);
    }

    #[test]
    fn scalar_range_normalizes_reversed_bounds() {
        assert_eq!(ScalarRange { min: 6.0, max: 2.0 }.sample(0.0), 2.0);
    }

    #[test]
    fn vec2_range_samples_components() {
        let value = Vec2Range {
            min: Vec2::new(0.0, 10.0),
            max: Vec2::new(10.0, 20.0),
        }
        .sample(0.25, 0.5);

        assert_eq!(value, Vec2::new(2.5, 15.0));
    }
}
