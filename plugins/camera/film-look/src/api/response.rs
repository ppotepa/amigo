#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FilmLookResponse2d {
    pub enabled: bool,
    pub grain: f32,
    pub halation: f32,
    pub sensor_response: f32,
    pub film_response: f32,
    pub tone_curve: f32,
}

impl Default for FilmLookResponse2d {
    fn default() -> Self {
        Self {
            enabled: false,
            grain: 0.0,
            halation: 0.0,
            sensor_response: 1.0,
            film_response: 1.0,
            tone_curve: 1.0,
        }
    }
}

impl FilmLookResponse2d {
    pub fn normalized(self) -> Self {
        Self {
            enabled: self.enabled,
            grain: finite_or_zero(self.grain).clamp(0.0, 4.0),
            halation: finite_or_zero(self.halation).clamp(0.0, 4.0),
            sensor_response: finite_or_zero(self.sensor_response).clamp(0.0, 8.0),
            film_response: finite_or_zero(self.film_response).clamp(0.0, 8.0),
            tone_curve: finite_or_zero(self.tone_curve).clamp(0.0, 8.0),
        }
    }

    pub fn is_enabled(self) -> bool {
        self.normalized().enabled
    }
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}
